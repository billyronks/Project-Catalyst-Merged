// Package gateway provides a Hasura-style unified API engine
// Auto-generates GraphQL, REST, gRPC, WebSocket, and MCP endpoints from LumaDB schema
package gateway

import (
	"context"
	"database/sql"
	"encoding/json"
	"fmt"
	"net/http"
	"strings"
	"sync"
	"time"

	"github.com/go-chi/chi/v5"
	"github.com/go-chi/chi/v5/middleware"
	"github.com/gorilla/websocket"
	"github.com/graphql-go/graphql"
	"github.com/rs/cors"
	"go.uber.org/zap"

	lumadb "github.com/brivas/unified-platform/packages/lumadb-client"
)

// UnifiedAPIEngine is the Hasura-style API engine that auto-generates APIs
type UnifiedAPIEngine struct {
	db           *lumadb.Client
	schema       *Schema
	graphqlAPI   *GraphQLHandler
	restAPI      *RESTHandler
	websocketAPI *WebSocketHandler
	mcpAPI       *MCPHandler
	router       chi.Router
	logger       *zap.Logger
	mu           sync.RWMutex
}

// Schema represents the database schema used for API generation
type Schema struct {
	Tables      []TableSchema            `json:"tables"`
	Permissions map[string]PermissionSet `json:"permissions"`
}

// TableSchema defines a table structure for API generation
type TableSchema struct {
	Name       string     `json:"name"`
	PrimaryKey string     `json:"primary_key"`
	Columns    []Column   `json:"columns"`
	Indexes    []Index    `json:"indexes"`
	Relations  []Relation `json:"relations"`
}

// Column represents a database column
type Column struct {
	Name     string `json:"name"`
	Type     string `json:"type"`
	Nullable bool   `json:"nullable"`
	Default  string `json:"default,omitempty"`
}

// Index represents a database index
type Index struct {
	Name    string   `json:"name"`
	Columns []string `json:"columns"`
	Unique  bool     `json:"unique"`
}

// Relation defines a foreign key relationship
type Relation struct {
	Name        string `json:"name"`
	Type        string `json:"type"` // one-to-one, one-to-many, many-to-many
	TargetTable string `json:"target_table"`
	ForeignKey  string `json:"foreign_key"`
	TargetKey   string `json:"target_key"`
}

// PermissionSet defines role-based permissions for a table
type PermissionSet struct {
	Select *Permission `json:"select,omitempty"`
	Insert *Permission `json:"insert,omitempty"`
	Update *Permission `json:"update,omitempty"`
	Delete *Permission `json:"delete,omitempty"`
}

// Permission defines access control rules
type Permission struct {
	Allowed bool              `json:"allowed"`
	Filter  map[string]string `json:"filter,omitempty"`  // Row-level security filter
	Columns []string          `json:"columns,omitempty"` // Allowed columns
	Check   map[string]string `json:"check,omitempty"`   // Insert/Update validation
}

// Config holds API gateway configuration
type Config struct {
	Port            int
	Host            string
	EnableGraphQL   bool
	EnableREST      bool
	EnableWebSocket bool
	EnableMCP       bool
	EnableCORS      bool
	AllowedOrigins  []string
}

// DefaultConfig returns default gateway configuration
func DefaultConfig() *Config {
	return &Config{
		Port:            8080,
		Host:            "0.0.0.0",
		EnableGraphQL:   true,
		EnableREST:      true,
		EnableWebSocket: true,
		EnableMCP:       true,
		EnableCORS:      true,
		AllowedOrigins:  []string{"*"},
	}
}

// NewUnifiedAPIEngine creates a new Hasura-style API engine
func NewUnifiedAPIEngine(db *lumadb.Client, logger *zap.Logger) *UnifiedAPIEngine {
	engine := &UnifiedAPIEngine{
		db:     db,
		logger: logger,
		router: chi.NewRouter(),
	}

	// Setup middleware
	engine.router.Use(middleware.RequestID)
	engine.router.Use(middleware.RealIP)
	engine.router.Use(middleware.Logger)
	engine.router.Use(middleware.Recoverer)
	engine.router.Use(middleware.Timeout(60 * time.Second))

	return engine
}

// LoadSchemaFromDB introspects LumaDB and builds schema for API generation
func (e *UnifiedAPIEngine) LoadSchemaFromDB(ctx context.Context) error {
	e.mu.Lock()
	defer e.mu.Unlock()

	schema := &Schema{
		Tables:      make([]TableSchema, 0),
		Permissions: make(map[string]PermissionSet),
	}

	// Query LumaDB information_schema for tables
	rows, err := e.db.Query(ctx, `
		SELECT table_name 
		FROM information_schema.tables 
		WHERE table_schema = 'public' AND table_type = 'BASE TABLE'
	`)
	if err != nil {
		return fmt.Errorf("failed to query tables: %w", err)
	}
	defer rows.Close()

	var tableNames []string
	for rows.Next() {
		var tableName string
		if err := rows.Scan(&tableName); err != nil {
			return fmt.Errorf("failed to scan table name: %w", err)
		}
		tableNames = append(tableNames, tableName)
	}

	// For each table, get column information
	for _, tableName := range tableNames {
		table := TableSchema{
			Name:    tableName,
			Columns: make([]Column, 0),
		}

		colRows, err := e.db.Query(ctx, `
			SELECT column_name, data_type, is_nullable, column_default
			FROM information_schema.columns
			WHERE table_name = $1 AND table_schema = 'public'
			ORDER BY ordinal_position
		`, tableName)
		if err != nil {
			e.logger.Warn("failed to get columns", zap.String("table", tableName), zap.Error(err))
			continue
		}

		for colRows.Next() {
			var col Column
			var nullable, defaultVal *string
			if err := colRows.Scan(&col.Name, &col.Type, &nullable, &defaultVal); err != nil {
				continue
			}
			col.Nullable = nullable != nil && *nullable == "YES"
			if defaultVal != nil {
				col.Default = *defaultVal
			}
			table.Columns = append(table.Columns, col)
		}
		colRows.Close()

		// Get primary key
		pkRow := e.db.QueryRow(ctx, `
			SELECT a.attname
			FROM pg_index i
			JOIN pg_attribute a ON a.attrelid = i.indrelid AND a.attnum = ANY(i.indkey)
			WHERE i.indrelid = $1::regclass AND i.indisprimary
		`, tableName)
		var pk string
		if err := pkRow.Scan(&pk); err == nil {
			table.PrimaryKey = pk
		} else {
			table.PrimaryKey = "id" // Default assumption
		}

		schema.Tables = append(schema.Tables, table)
	}

	e.schema = schema
	e.logger.Info("schema loaded", zap.Int("tables", len(schema.Tables)))

	return nil
}

// GenerateAPIs generates all API endpoints from the loaded schema
func (e *UnifiedAPIEngine) GenerateAPIs(cfg *Config) error {
	if e.schema == nil {
		return fmt.Errorf("schema not loaded, call LoadSchemaFromDB first")
	}

	// Generate GraphQL API
	if cfg.EnableGraphQL {
		e.graphqlAPI = NewGraphQLHandler(e.db, e.schema, e.logger)
		e.router.Handle("/graphql", e.graphqlAPI)
		e.router.Handle("/v1/graphql", e.graphqlAPI) // Hasura-compatible path
		e.logger.Info("GraphQL API enabled", zap.String("path", "/graphql"))
	}

	// Generate REST API
	if cfg.EnableREST {
		e.restAPI = NewRESTHandler(e.db, e.schema, e.logger)
		e.router.Mount("/api/v1", e.restAPI.Routes())
		e.logger.Info("REST API enabled", zap.String("path", "/api/v1"))
	}

	// Generate WebSocket API for subscriptions
	if cfg.EnableWebSocket {
		e.websocketAPI = NewWebSocketHandler(e.db, e.schema, e.logger)
		e.router.Handle("/ws", e.websocketAPI)
		e.logger.Info("WebSocket API enabled", zap.String("path", "/ws"))
	}

	// Generate MCP API for LLM integration
	if cfg.EnableMCP {
		e.mcpAPI = NewMCPHandler(e.db, e.schema, e.logger)
		e.router.Mount("/mcp", e.mcpAPI.Routes())
		e.logger.Info("MCP API enabled", zap.String("path", "/mcp"))
	}

	// Health check
	e.router.Get("/health", e.healthCheck)
	e.router.Get("/ready", e.readinessCheck)

	return nil
}

// Start starts the API server
func (e *UnifiedAPIEngine) Start(cfg *Config) error {
	handler := e.router

	// Enable CORS if configured
	if cfg.EnableCORS {
		c := cors.New(cors.Options{
			AllowedOrigins:   cfg.AllowedOrigins,
			AllowedMethods:   []string{"GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS"},
			AllowedHeaders:   []string{"*"},
			AllowCredentials: true,
		})
		handler = c.Handler(e.router).(chi.Router)
	}

	addr := fmt.Sprintf("%s:%d", cfg.Host, cfg.Port)
	e.logger.Info("starting unified API gateway", zap.String("addr", addr))

	return http.ListenAndServe(addr, handler)
}

func (e *UnifiedAPIEngine) healthCheck(w http.ResponseWriter, r *http.Request) {
	ctx := r.Context()

	health := map[string]interface{}{
		"status":    "healthy",
		"timestamp": time.Now().UTC(),
		"version":   "1.0.0",
	}

	// Check database connection
	if err := e.db.Health(ctx); err != nil {
		health["status"] = "unhealthy"
		health["database"] = "disconnected"
		w.WriteHeader(http.StatusServiceUnavailable)
	} else {
		health["database"] = "connected"
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(health)
}

func (e *UnifiedAPIEngine) readinessCheck(w http.ResponseWriter, r *http.Request) {
	ready := map[string]interface{}{
		"ready":  e.schema != nil && len(e.schema.Tables) > 0,
		"tables": 0,
	}
	if e.schema != nil {
		ready["tables"] = len(e.schema.Tables)
	}

	w.Header().Set("Content-Type", "application/json")
	if !ready["ready"].(bool) {
		w.WriteHeader(http.StatusServiceUnavailable)
	}
	json.NewEncoder(w).Encode(ready)
}

// GraphQLHandler handles GraphQL requests
type GraphQLHandler struct {
	db     *lumadb.Client
	schema *graphql.Schema
	logger *zap.Logger
}

// NewGraphQLHandler creates a new GraphQL handler with auto-generated schema
func NewGraphQLHandler(db *lumadb.Client, dbSchema *Schema, logger *zap.Logger) *GraphQLHandler {
	handler := &GraphQLHandler{
		db:     db,
		logger: logger,
	}

	// Build GraphQL schema from database schema
	queryFields := graphql.Fields{}
	mutationFields := graphql.Fields{}

	for _, table := range dbSchema.Tables {
		tableName := table.Name
		objType := handler.buildObjectType(table)

		// Generate query: get single record
		queryFields[toCamelCase(tableName)] = &graphql.Field{
			Type: objType,
			Args: graphql.FieldConfigArgument{
				"id": &graphql.ArgumentConfig{Type: graphql.NewNonNull(graphql.ID)},
			},
			Resolve: handler.resolveGetOne(tableName, table.PrimaryKey),
		}

		// Generate query: list records
		queryFields[toPlural(toCamelCase(tableName))] = &graphql.Field{
			Type: graphql.NewList(objType),
			Args: graphql.FieldConfigArgument{
				"where":   &graphql.ArgumentConfig{Type: graphql.String},
				"limit":   &graphql.ArgumentConfig{Type: graphql.Int},
				"offset":  &graphql.ArgumentConfig{Type: graphql.Int},
				"orderBy": &graphql.ArgumentConfig{Type: graphql.String},
			},
			Resolve: handler.resolveList(tableName),
		}

		// Generate mutation: insert
		mutationFields["insert_"+tableName] = &graphql.Field{
			Type: objType,
			Args: graphql.FieldConfigArgument{
				"object": &graphql.ArgumentConfig{Type: graphql.NewNonNull(graphql.String)},
			},
			Resolve: handler.resolveInsert(tableName),
		}

		// Generate mutation: update
		mutationFields["update_"+tableName] = &graphql.Field{
			Type: objType,
			Args: graphql.FieldConfigArgument{
				"id":   &graphql.ArgumentConfig{Type: graphql.NewNonNull(graphql.ID)},
				"_set": &graphql.ArgumentConfig{Type: graphql.NewNonNull(graphql.String)},
			},
			Resolve: handler.resolveUpdate(tableName, table.PrimaryKey),
		}

		// Generate mutation: delete
		mutationFields["delete_"+tableName] = &graphql.Field{
			Type: objType,
			Args: graphql.FieldConfigArgument{
				"id": &graphql.ArgumentConfig{Type: graphql.NewNonNull(graphql.ID)},
			},
			Resolve: handler.resolveDelete(tableName, table.PrimaryKey),
		}
	}

	queryType := graphql.NewObject(graphql.ObjectConfig{
		Name:   "Query",
		Fields: queryFields,
	})

	mutationType := graphql.NewObject(graphql.ObjectConfig{
		Name:   "Mutation",
		Fields: mutationFields,
	})

	schema, err := graphql.NewSchema(graphql.SchemaConfig{
		Query:    queryType,
		Mutation: mutationType,
	})
	if err != nil {
		logger.Error("failed to create GraphQL schema", zap.Error(err))
	}
	handler.schema = &schema

	return handler
}

func (h *GraphQLHandler) buildObjectType(table TableSchema) *graphql.Object {
	fields := graphql.Fields{}

	for _, col := range table.Columns {
		fields[toCamelCase(col.Name)] = &graphql.Field{
			Type: mapSQLTypeToGraphQL(col.Type),
		}
	}

	return graphql.NewObject(graphql.ObjectConfig{
		Name:   toPascalCase(table.Name),
		Fields: fields,
	})
}

func (h *GraphQLHandler) resolveGetOne(tableName, primaryKey string) graphql.FieldResolveFn {
	return func(p graphql.ResolveParams) (interface{}, error) {
		id := p.Args["id"]
		query := fmt.Sprintf("SELECT * FROM %s WHERE %s = $1", tableName, primaryKey)

		row := h.db.QueryRow(p.Context, query, id)
		// Scan into map - simplified for this example
		return scanRowToMap(row, nil)
	}
}

func (h *GraphQLHandler) resolveList(tableName string) graphql.FieldResolveFn {
	return func(p graphql.ResolveParams) (interface{}, error) {
		query := fmt.Sprintf("SELECT * FROM %s", tableName)

		var args []interface{}
		argIdx := 1

		if where, ok := p.Args["where"].(string); ok && where != "" {
			query += " WHERE " + where
		}

		if orderBy, ok := p.Args["orderBy"].(string); ok && orderBy != "" {
			query += " ORDER BY " + orderBy
		}

		if limit, ok := p.Args["limit"].(int); ok {
			query += fmt.Sprintf(" LIMIT $%d", argIdx)
			args = append(args, limit)
			argIdx++
		}

		if offset, ok := p.Args["offset"].(int); ok {
			query += fmt.Sprintf(" OFFSET $%d", argIdx)
			args = append(args, offset)
		}

		rows, err := h.db.Query(p.Context, query, args...)
		if err != nil {
			return nil, err
		}
		defer rows.Close()

		return scanRowsToMaps(rows)
	}
}

func (h *GraphQLHandler) resolveInsert(tableName string) graphql.FieldResolveFn {
	return func(p graphql.ResolveParams) (interface{}, error) {
		objectJSON := p.Args["object"].(string)
		var data map[string]interface{}
		if err := json.Unmarshal([]byte(objectJSON), &data); err != nil {
			return nil, err
		}

		columns := make([]string, 0, len(data))
		placeholders := make([]string, 0, len(data))
		values := make([]interface{}, 0, len(data))

		i := 1
		for col, val := range data {
			columns = append(columns, col)
			placeholders = append(placeholders, fmt.Sprintf("$%d", i))
			values = append(values, val)
			i++
		}

		query := fmt.Sprintf(
			"INSERT INTO %s (%s) VALUES (%s) RETURNING *",
			tableName,
			strings.Join(columns, ", "),
			strings.Join(placeholders, ", "),
		)

		row := h.db.QueryRow(p.Context, query, values...)
		return scanRowToMap(row, nil)
	}
}

func (h *GraphQLHandler) resolveUpdate(tableName, primaryKey string) graphql.FieldResolveFn {
	return func(p graphql.ResolveParams) (interface{}, error) {
		id := p.Args["id"]
		setJSON := p.Args["_set"].(string)

		var data map[string]interface{}
		if err := json.Unmarshal([]byte(setJSON), &data); err != nil {
			return nil, err
		}

		setClauses := make([]string, 0, len(data))
		values := make([]interface{}, 0, len(data)+1)

		i := 1
		for col, val := range data {
			setClauses = append(setClauses, fmt.Sprintf("%s = $%d", col, i))
			values = append(values, val)
			i++
		}
		values = append(values, id)

		query := fmt.Sprintf(
			"UPDATE %s SET %s WHERE %s = $%d RETURNING *",
			tableName,
			strings.Join(setClauses, ", "),
			primaryKey,
			i,
		)

		row := h.db.QueryRow(p.Context, query, values...)
		return scanRowToMap(row, nil)
	}
}

func (h *GraphQLHandler) resolveDelete(tableName, primaryKey string) graphql.FieldResolveFn {
	return func(p graphql.ResolveParams) (interface{}, error) {
		id := p.Args["id"]
		query := fmt.Sprintf("DELETE FROM %s WHERE %s = $1 RETURNING *", tableName, primaryKey)

		row := h.db.QueryRow(p.Context, query, id)
		return scanRowToMap(row, nil)
	}
}

func (h *GraphQLHandler) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	var params struct {
		Query         string                 `json:"query"`
		OperationName string                 `json:"operationName"`
		Variables     map[string]interface{} `json:"variables"`
	}

	if r.Method == "POST" {
		if err := json.NewDecoder(r.Body).Decode(&params); err != nil {
			http.Error(w, err.Error(), http.StatusBadRequest)
			return
		}
	} else {
		params.Query = r.URL.Query().Get("query")
	}

	result := graphql.Do(graphql.Params{
		Schema:         *h.schema,
		RequestString:  params.Query,
		VariableValues: params.Variables,
		OperationName:  params.OperationName,
		Context:        r.Context(),
	})

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(result)
}

// RESTHandler handles REST API requests
type RESTHandler struct {
	db     *lumadb.Client
	schema *Schema
	logger *zap.Logger
}

// NewRESTHandler creates a new REST handler
func NewRESTHandler(db *lumadb.Client, schema *Schema, logger *zap.Logger) *RESTHandler {
	return &RESTHandler{db: db, schema: schema, logger: logger}
}

// Routes returns the REST API routes
func (h *RESTHandler) Routes() chi.Router {
	r := chi.NewRouter()

	for _, table := range h.schema.Tables {
		tableName := table.Name
		pk := table.PrimaryKey

		// GET /resource - List
		r.Get("/"+tableName, h.handleList(tableName))

		// GET /resource/{id} - Get one
		r.Get("/"+tableName+"/{id}", h.handleGetOne(tableName, pk))

		// POST /resource - Create
		r.Post("/"+tableName, h.handleCreate(tableName))

		// PUT /resource/{id} - Update
		r.Put("/"+tableName+"/{id}", h.handleUpdate(tableName, pk))

		// PATCH /resource/{id} - Partial update
		r.Patch("/"+tableName+"/{id}", h.handleUpdate(tableName, pk))

		// DELETE /resource/{id} - Delete
		r.Delete("/"+tableName+"/{id}", h.handleDelete(tableName, pk))

		// POST /resource/bulk - Bulk insert
		r.Post("/"+tableName+"/bulk", h.handleBulkCreate(tableName))
	}

	return r
}

func (h *RESTHandler) handleList(tableName string) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		ctx := r.Context()
		query := fmt.Sprintf("SELECT * FROM %s LIMIT 100", tableName)

		rows, err := h.db.Query(ctx, query)
		if err != nil {
			h.jsonError(w, err.Error(), http.StatusInternalServerError)
			return
		}
		defer rows.Close()

		results, err := scanRowsToMaps(rows)
		if err != nil {
			h.jsonError(w, err.Error(), http.StatusInternalServerError)
			return
		}

		h.jsonResponse(w, results, http.StatusOK)
	}
}

func (h *RESTHandler) handleGetOne(tableName, pk string) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		ctx := r.Context()
		id := chi.URLParam(r, "id")

		query := fmt.Sprintf("SELECT * FROM %s WHERE %s = $1", tableName, pk)
		row := h.db.QueryRow(ctx, query, id)

		result, err := scanRowToMap(row, nil)
		if err != nil {
			h.jsonError(w, "not found", http.StatusNotFound)
			return
		}

		h.jsonResponse(w, result, http.StatusOK)
	}
}

func (h *RESTHandler) handleCreate(tableName string) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		ctx := r.Context()

		var data map[string]interface{}
		if err := json.NewDecoder(r.Body).Decode(&data); err != nil {
			h.jsonError(w, "invalid JSON", http.StatusBadRequest)
			return
		}

		columns := make([]string, 0, len(data))
		placeholders := make([]string, 0, len(data))
		values := make([]interface{}, 0, len(data))

		i := 1
		for col, val := range data {
			columns = append(columns, col)
			placeholders = append(placeholders, fmt.Sprintf("$%d", i))
			values = append(values, val)
			i++
		}

		query := fmt.Sprintf(
			"INSERT INTO %s (%s) VALUES (%s) RETURNING *",
			tableName,
			strings.Join(columns, ", "),
			strings.Join(placeholders, ", "),
		)

		row := h.db.QueryRow(ctx, query, values...)
		result, err := scanRowToMap(row, nil)
		if err != nil {
			h.jsonError(w, err.Error(), http.StatusInternalServerError)
			return
		}

		h.jsonResponse(w, result, http.StatusCreated)
	}
}

func (h *RESTHandler) handleUpdate(tableName, pk string) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		ctx := r.Context()
		id := chi.URLParam(r, "id")

		var data map[string]interface{}
		if err := json.NewDecoder(r.Body).Decode(&data); err != nil {
			h.jsonError(w, "invalid JSON", http.StatusBadRequest)
			return
		}

		setClauses := make([]string, 0, len(data))
		values := make([]interface{}, 0, len(data)+1)

		i := 1
		for col, val := range data {
			setClauses = append(setClauses, fmt.Sprintf("%s = $%d", col, i))
			values = append(values, val)
			i++
		}
		values = append(values, id)

		query := fmt.Sprintf(
			"UPDATE %s SET %s WHERE %s = $%d RETURNING *",
			tableName,
			strings.Join(setClauses, ", "),
			pk,
			i,
		)

		row := h.db.QueryRow(ctx, query, values...)
		result, err := scanRowToMap(row, nil)
		if err != nil {
			h.jsonError(w, "not found", http.StatusNotFound)
			return
		}

		h.jsonResponse(w, result, http.StatusOK)
	}
}

func (h *RESTHandler) handleDelete(tableName, pk string) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		ctx := r.Context()
		id := chi.URLParam(r, "id")

		query := fmt.Sprintf("DELETE FROM %s WHERE %s = $1 RETURNING *", tableName, pk)
		row := h.db.QueryRow(ctx, query, id)

		result, err := scanRowToMap(row, nil)
		if err != nil {
			h.jsonError(w, "not found", http.StatusNotFound)
			return
		}

		h.jsonResponse(w, result, http.StatusOK)
	}
}

func (h *RESTHandler) handleBulkCreate(tableName string) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		ctx := r.Context()

		var items []map[string]interface{}
		if err := json.NewDecoder(r.Body).Decode(&items); err != nil {
			h.jsonError(w, "invalid JSON array", http.StatusBadRequest)
			return
		}

		results := make([]map[string]interface{}, 0, len(items))

		for _, data := range items {
			columns := make([]string, 0, len(data))
			placeholders := make([]string, 0, len(data))
			values := make([]interface{}, 0, len(data))

			i := 1
			for col, val := range data {
				columns = append(columns, col)
				placeholders = append(placeholders, fmt.Sprintf("$%d", i))
				values = append(values, val)
				i++
			}

			query := fmt.Sprintf(
				"INSERT INTO %s (%s) VALUES (%s) RETURNING *",
				tableName,
				strings.Join(columns, ", "),
				strings.Join(placeholders, ", "),
			)

			row := h.db.QueryRow(ctx, query, values...)
			result, err := scanRowToMap(row, nil)
			if err != nil {
				continue
			}
			results = append(results, result)
		}

		h.jsonResponse(w, map[string]interface{}{
			"inserted": len(results),
			"data":     results,
		}, http.StatusCreated)
	}
}

func (h *RESTHandler) jsonResponse(w http.ResponseWriter, data interface{}, status int) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	json.NewEncoder(w).Encode(data)
}

func (h *RESTHandler) jsonError(w http.ResponseWriter, message string, status int) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	json.NewEncoder(w).Encode(map[string]string{"error": message})
}

// WebSocketHandler handles WebSocket subscriptions
type WebSocketHandler struct {
	db       *lumadb.Client
	schema   *Schema
	logger   *zap.Logger
	upgrader websocket.Upgrader
	clients  sync.Map
}

// NewWebSocketHandler creates a new WebSocket handler
func NewWebSocketHandler(db *lumadb.Client, schema *Schema, logger *zap.Logger) *WebSocketHandler {
	return &WebSocketHandler{
		db:     db,
		schema: schema,
		logger: logger,
		upgrader: websocket.Upgrader{
			CheckOrigin: func(r *http.Request) bool { return true },
		},
	}
}

func (h *WebSocketHandler) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	conn, err := h.upgrader.Upgrade(w, r, nil)
	if err != nil {
		h.logger.Error("websocket upgrade failed", zap.Error(err))
		return
	}
	defer conn.Close()

	clientID := fmt.Sprintf("%p", conn)
	h.clients.Store(clientID, conn)
	defer h.clients.Delete(clientID)

	for {
		_, message, err := conn.ReadMessage()
		if err != nil {
			break
		}

		var msg struct {
			Type    string          `json:"type"`
			Channel string          `json:"channel"`
			Payload json.RawMessage `json:"payload"`
		}

		if err := json.Unmarshal(message, &msg); err != nil {
			continue
		}

		switch msg.Type {
		case "subscribe":
			h.logger.Info("client subscribed", zap.String("channel", msg.Channel))
		case "unsubscribe":
			h.logger.Info("client unsubscribed", zap.String("channel", msg.Channel))
		}
	}
}

// MCPHandler handles Model Context Protocol requests for LLM integration
type MCPHandler struct {
	db     *lumadb.Client
	schema *Schema
	logger *zap.Logger
	tools  map[string]MCPTool
}

// MCPTool defines an MCP tool for LLM consumption
type MCPTool struct {
	Name        string                                                             `json:"name"`
	Description string                                                             `json:"description"`
	InputSchema map[string]interface{}                                             `json:"input_schema"`
	Handler     func(context.Context, map[string]interface{}) (interface{}, error) `json:"-"`
}

// NewMCPHandler creates a new MCP handler
func NewMCPHandler(db *lumadb.Client, schema *Schema, logger *zap.Logger) *MCPHandler {
	h := &MCPHandler{
		db:     db,
		schema: schema,
		logger: logger,
		tools:  make(map[string]MCPTool),
	}

	// Generate MCP tools for each table
	for _, table := range schema.Tables {
		h.registerTableTools(table)
	}

	return h
}

func (h *MCPHandler) registerTableTools(table TableSchema) {
	tableName := table.Name

	// List tool
	h.tools["list_"+tableName] = MCPTool{
		Name:        "list_" + tableName,
		Description: fmt.Sprintf("List %s records with optional filters", tableName),
		InputSchema: map[string]interface{}{
			"type": "object",
			"properties": map[string]interface{}{
				"limit":  map[string]string{"type": "integer", "description": "Maximum records to return"},
				"offset": map[string]string{"type": "integer", "description": "Number of records to skip"},
			},
		},
		Handler: func(ctx context.Context, input map[string]interface{}) (interface{}, error) {
			limit := 100
			if l, ok := input["limit"].(float64); ok {
				limit = int(l)
			}

			query := fmt.Sprintf("SELECT * FROM %s LIMIT %d", tableName, limit)
			rows, err := h.db.Query(ctx, query)
			if err != nil {
				return nil, err
			}
			defer rows.Close()
			return scanRowsToMaps(rows)
		},
	}

	// Get tool
	h.tools["get_"+tableName] = MCPTool{
		Name:        "get_" + tableName,
		Description: fmt.Sprintf("Get a single %s record by ID", tableName),
		InputSchema: map[string]interface{}{
			"type": "object",
			"properties": map[string]interface{}{
				"id": map[string]string{"type": "string", "description": "Record ID"},
			},
			"required": []string{"id"},
		},
		Handler: func(ctx context.Context, input map[string]interface{}) (interface{}, error) {
			id := input["id"]
			query := fmt.Sprintf("SELECT * FROM %s WHERE %s = $1", tableName, table.PrimaryKey)
			row := h.db.QueryRow(ctx, query, id)
			return scanRowToMap(row, nil)
		},
	}
}

// Routes returns the MCP API routes
func (h *MCPHandler) Routes() chi.Router {
	r := chi.NewRouter()

	// List available tools
	r.Get("/tools", func(w http.ResponseWriter, r *http.Request) {
		tools := make([]map[string]interface{}, 0, len(h.tools))
		for _, tool := range h.tools {
			tools = append(tools, map[string]interface{}{
				"name":         tool.Name,
				"description":  tool.Description,
				"input_schema": tool.InputSchema,
			})
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(map[string]interface{}{"tools": tools})
	})

	// Execute tool
	r.Post("/tools/{name}/execute", func(w http.ResponseWriter, r *http.Request) {
		toolName := chi.URLParam(r, "name")
		tool, ok := h.tools[toolName]
		if !ok {
			http.Error(w, "tool not found", http.StatusNotFound)
			return
		}

		var input map[string]interface{}
		if err := json.NewDecoder(r.Body).Decode(&input); err != nil {
			http.Error(w, "invalid input", http.StatusBadRequest)
			return
		}

		result, err := tool.Handler(r.Context(), input)
		if err != nil {
			http.Error(w, err.Error(), http.StatusInternalServerError)
			return
		}

		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(map[string]interface{}{"result": result})
	})

	return r
}

// Helper functions

func mapSQLTypeToGraphQL(sqlType string) graphql.Output {
	switch strings.ToLower(sqlType) {
	case "integer", "int", "smallint", "bigint", "serial":
		return graphql.Int
	case "real", "double precision", "numeric", "decimal":
		return graphql.Float
	case "boolean", "bool":
		return graphql.Boolean
	case "json", "jsonb":
		return graphql.String // JSON as string for simplicity
	default:
		return graphql.String
	}
}

func toCamelCase(s string) string {
	parts := strings.Split(s, "_")
	for i := 1; i < len(parts); i++ {
		parts[i] = strings.Title(parts[i])
	}
	return strings.Join(parts, "")
}

func toPascalCase(s string) string {
	parts := strings.Split(s, "_")
	for i := 0; i < len(parts); i++ {
		parts[i] = strings.Title(parts[i])
	}
	return strings.Join(parts, "")
}

func toPlural(s string) string {
	if strings.HasSuffix(s, "s") {
		return s + "es"
	}
	if strings.HasSuffix(s, "y") {
		return s[:len(s)-1] + "ies"
	}
	return s + "s"
}

func scanRowToMap(row *sql.Row, cols []string) (map[string]interface{}, error) {
	// This is a simplified implementation
	// In production, use sqlx or implement proper column scanning
	result := make(map[string]interface{})
	// Implementation would scan row into result map
	return result, nil
}

func scanRowsToMaps(rows *sql.Rows) ([]map[string]interface{}, error) {
	cols, err := rows.Columns()
	if err != nil {
		return nil, err
	}

	results := make([]map[string]interface{}, 0)

	for rows.Next() {
		columns := make([]interface{}, len(cols))
		columnPointers := make([]interface{}, len(cols))
		for i := range columns {
			columnPointers[i] = &columns[i]
		}

		if err := rows.Scan(columnPointers...); err != nil {
			return nil, err
		}

		m := make(map[string]interface{})
		for i, colName := range cols {
			val := columns[i]
			if b, ok := val.([]byte); ok {
				m[colName] = string(b)
			} else {
				m[colName] = val
			}
		}
		results = append(results, m)
	}

	return results, nil
}
