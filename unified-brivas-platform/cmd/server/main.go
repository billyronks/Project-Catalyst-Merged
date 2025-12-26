// Unified Brivas Platform - Main Entry Point
package main

import (
	"context"
	"fmt"
	"os"
	"os/signal"
	"syscall"
	"time"

	"go.uber.org/zap"

	gateway "github.com/brivas/unified-platform/apps/api-gateway"
	lumadb "github.com/brivas/unified-platform/packages/lumadb-client"
)

func main() {
	// Initialize logger
	logger, _ := zap.NewProduction()
	defer logger.Sync()

	logger.Info("Starting Unified Brivas Platform",
		zap.String("version", "1.0.0"),
		zap.Time("startup", time.Now()),
	)

	// Load configuration from environment
	dbConfig := &lumadb.Config{
		Host:            getEnv("LUMADB_HOST", "localhost"),
		Port:            getEnvInt("LUMADB_PORT", 5432),
		Database:        getEnv("LUMADB_DATABASE", "brivas"),
		User:            getEnv("LUMADB_USER", "brivas"),
		Password:        getEnv("LUMADB_PASSWORD", ""),
		SSLMode:         getEnv("LUMADB_SSLMODE", "disable"),
		MaxOpenConns:    getEnvInt("LUMADB_MAX_OPEN_CONNS", 100),
		MaxIdleConns:    getEnvInt("LUMADB_MAX_IDLE_CONNS", 25),
		ConnMaxLifetime: 5 * time.Minute,
		ConnMaxIdleTime: 1 * time.Minute,
	}

	// Connect to LumaDB
	db, err := lumadb.Connect(dbConfig)
	if err != nil {
		logger.Fatal("Failed to connect to LumaDB", zap.Error(err))
	}
	defer db.Close()

	logger.Info("Connected to LumaDB",
		zap.String("host", dbConfig.Host),
		zap.Int("port", dbConfig.Port),
		zap.String("database", dbConfig.Database),
	)

	// Create API engine
	engine := gateway.NewUnifiedAPIEngine(db, logger)

	// Load schema from database
	ctx := context.Background()
	if err := engine.LoadSchemaFromDB(ctx); err != nil {
		logger.Fatal("Failed to load schema from LumaDB", zap.Error(err))
	}

	// Configure and generate APIs
	apiConfig := &gateway.Config{
		Port:            getEnvInt("API_PORT", 8080),
		Host:            getEnv("API_HOST", "0.0.0.0"),
		EnableGraphQL:   getEnvBool("ENABLE_GRAPHQL", true),
		EnableREST:      getEnvBool("ENABLE_REST", true),
		EnableWebSocket: getEnvBool("ENABLE_WEBSOCKET", true),
		EnableMCP:       getEnvBool("ENABLE_MCP", true),
		EnableCORS:      getEnvBool("ENABLE_CORS", true),
		AllowedOrigins:  []string{"*"},
	}

	if err := engine.GenerateAPIs(apiConfig); err != nil {
		logger.Fatal("Failed to generate APIs", zap.Error(err))
	}

	// Graceful shutdown handling
	shutdown := make(chan os.Signal, 1)
	signal.Notify(shutdown, os.Interrupt, syscall.SIGTERM)

	go func() {
		if err := engine.Start(apiConfig); err != nil {
			logger.Fatal("API server failed", zap.Error(err))
		}
	}()

	logger.Info("Unified Brivas Platform started",
		zap.Int("port", apiConfig.Port),
		zap.Bool("graphql", apiConfig.EnableGraphQL),
		zap.Bool("rest", apiConfig.EnableREST),
		zap.Bool("websocket", apiConfig.EnableWebSocket),
		zap.Bool("mcp", apiConfig.EnableMCP),
	)

	<-shutdown
	logger.Info("Shutting down...")
}

func getEnv(key, defaultValue string) string {
	if value := os.Getenv(key); value != "" {
		return value
	}
	return defaultValue
}

func getEnvInt(key string, defaultValue int) int {
	if value := os.Getenv(key); value != "" {
		var result int
		if _, err := fmt.Sscanf(value, "%d", &result); err == nil {
			return result
		}
	}
	return defaultValue
}

func getEnvBool(key string, defaultValue bool) bool {
	if value := os.Getenv(key); value != "" {
		return value == "true" || value == "1" || value == "yes"
	}
	return defaultValue
}
