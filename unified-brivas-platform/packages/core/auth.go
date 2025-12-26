// Package auth provides Hasura-style role-based access control (RBAC)
// with row-level security for the unified API gateway
package auth

import (
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"strings"
	"time"

	"github.com/golang-jwt/jwt/v5"
	"go.uber.org/zap"

	lumadb "github.com/brivas/unified-platform/packages/lumadb-client"
)

// Role defines user roles in the system
type Role string

const (
	RoleAnonymous  Role = "anonymous"
	RoleUser       Role = "user"
	RoleAdmin      Role = "admin"
	RoleReseller   Role = "reseller"
	RoleService    Role = "service"
	RoleSuperAdmin Role = "super_admin"
)

// Permission defines CRUD operations
type Permission string

const (
	PermissionSelect Permission = "select"
	PermissionInsert Permission = "insert"
	PermissionUpdate Permission = "update"
	PermissionDelete Permission = "delete"
)

// Claims represents JWT claims for authentication
type Claims struct {
	jwt.RegisteredClaims
	AccountID   string                 `json:"account_id"`
	Role        Role                   `json:"role"`
	Permissions []string               `json:"permissions,omitempty"`
	IsLive      bool                   `json:"is_live"`
	Metadata    map[string]interface{} `json:"metadata,omitempty"`
}

// TablePermission defines permissions for a specific table
type TablePermission struct {
	Role   Role              `json:"role"`
	Table  string            `json:"table"`
	Select *SelectPermission `json:"select,omitempty"`
	Insert *InsertPermission `json:"insert,omitempty"`
	Update *UpdatePermission `json:"update,omitempty"`
	Delete *DeletePermission `json:"delete,omitempty"`
}

// SelectPermission defines row-level security for SELECT
type SelectPermission struct {
	Allowed bool              `json:"allowed"`
	Columns []string          `json:"columns,omitempty"`
	Filter  map[string]string `json:"filter,omitempty"`
	Limit   int               `json:"limit,omitempty"`
}

// InsertPermission defines constraints for INSERT
type InsertPermission struct {
	Allowed bool              `json:"allowed"`
	Columns []string          `json:"columns,omitempty"`
	Check   map[string]string `json:"check,omitempty"`
	Set     map[string]string `json:"set,omitempty"`
}

// UpdatePermission defines constraints for UPDATE
type UpdatePermission struct {
	Allowed bool              `json:"allowed"`
	Columns []string          `json:"columns,omitempty"`
	Filter  map[string]string `json:"filter,omitempty"`
}

// DeletePermission defines constraints for DELETE
type DeletePermission struct {
	Allowed bool              `json:"allowed"`
	Filter  map[string]string `json:"filter,omitempty"`
}

// AuthorizationEngine manages role-based access control
type AuthorizationEngine struct {
	db          *lumadb.Client
	logger      *zap.Logger
	jwtSecret   []byte
	permissions map[string]map[Role]*TablePermission
}

// NewAuthorizationEngine creates a new authorization engine
func NewAuthorizationEngine(db *lumadb.Client, jwtSecret string, logger *zap.Logger) *AuthorizationEngine {
	engine := &AuthorizationEngine{
		db:          db,
		logger:      logger,
		jwtSecret:   []byte(jwtSecret),
		permissions: make(map[string]map[Role]*TablePermission),
	}
	engine.initializeDefaultPermissions()
	return engine
}

// initializeDefaultPermissions sets up Hasura-style default RBAC rules
func (e *AuthorizationEngine) initializeDefaultPermissions() {
	// Accounts - Users see only their own data
	e.setPermission(&TablePermission{
		Role:  RoleUser,
		Table: "accounts",
		Select: &SelectPermission{
			Allowed: true,
			Columns: []string{"id", "email", "first_name", "last_name", "balance"},
			Filter:  map[string]string{"id": "X-Account-ID"},
		},
		Update: &UpdatePermission{
			Allowed: true,
			Columns: []string{"first_name", "last_name", "phone_number"},
			Filter:  map[string]string{"id": "X-Account-ID"},
		},
	})

	// Accounts - Admin has full access
	e.setPermission(&TablePermission{
		Role:   RoleAdmin,
		Table:  "accounts",
		Select: &SelectPermission{Allowed: true},
		Insert: &InsertPermission{Allowed: true},
		Update: &UpdatePermission{Allowed: true},
		Delete: &DeletePermission{Allowed: true},
	})

	// SMS History - Users see only their messages
	e.setPermission(&TablePermission{
		Role:  RoleUser,
		Table: "sms_history",
		Select: &SelectPermission{
			Allowed: true,
			Filter:  map[string]string{"account_id": "X-Account-ID"},
			Limit:   1000,
		},
		Insert: &InsertPermission{
			Allowed: true,
			Set:     map[string]string{"account_id": "X-Account-ID"},
		},
	})

	// Campaigns
	e.setPermission(&TablePermission{
		Role:  RoleUser,
		Table: "campaigns",
		Select: &SelectPermission{
			Allowed: true,
			Filter:  map[string]string{"account_id": "X-Account-ID"},
		},
		Insert: &InsertPermission{
			Allowed: true,
			Set:     map[string]string{"account_id": "X-Account-ID"},
		},
		Update: &UpdatePermission{
			Allowed: true,
			Filter:  map[string]string{"account_id": "X-Account-ID"},
		},
		Delete: &DeletePermission{
			Allowed: true,
			Filter:  map[string]string{"account_id": "X-Account-ID"},
		},
	})

	// Sender IDs
	e.setPermission(&TablePermission{
		Role:  RoleUser,
		Table: "sender_ids",
		Select: &SelectPermission{
			Allowed: true,
			Filter:  map[string]string{"account_id": "X-Account-ID"},
		},
		Insert: &InsertPermission{
			Allowed: true,
			Set:     map[string]string{"account_id": "X-Account-ID", "status": "'pending'"},
		},
	})

	// Billing
	e.setPermission(&TablePermission{
		Role:  RoleUser,
		Table: "billing_transactions",
		Select: &SelectPermission{
			Allowed: true,
			Filter:  map[string]string{"tenant_id": "X-Account-ID"},
		},
	})
}

func (e *AuthorizationEngine) setPermission(perm *TablePermission) {
	if _, ok := e.permissions[perm.Table]; !ok {
		e.permissions[perm.Table] = make(map[Role]*TablePermission)
	}
	e.permissions[perm.Table][perm.Role] = perm
}

// GetPermission returns the permission for a table and role
func (e *AuthorizationEngine) GetPermission(table string, role Role) *TablePermission {
	if tablePerms, ok := e.permissions[table]; ok {
		if perm, ok := tablePerms[role]; ok {
			return perm
		}
	}
	return nil
}

// GenerateToken generates a JWT token for a user
func (e *AuthorizationEngine) GenerateToken(accountID string, role Role, isLive bool) (string, error) {
	claims := &Claims{
		RegisteredClaims: jwt.RegisteredClaims{
			ExpiresAt: jwt.NewNumericDate(time.Now().Add(24 * time.Hour)),
			IssuedAt:  jwt.NewNumericDate(time.Now()),
			Issuer:    "brivas-platform",
		},
		AccountID: accountID,
		Role:      role,
		IsLive:    isLive,
	}
	token := jwt.NewWithClaims(jwt.SigningMethodHS256, claims)
	return token.SignedString(e.jwtSecret)
}

// ValidateToken validates a JWT token
func (e *AuthorizationEngine) ValidateToken(tokenString string) (*Claims, error) {
	token, err := jwt.ParseWithClaims(tokenString, &Claims{}, func(token *jwt.Token) (interface{}, error) {
		return e.jwtSecret, nil
	})
	if err != nil {
		return nil, err
	}
	if claims, ok := token.Claims.(*Claims); ok && token.Valid {
		return claims, nil
	}
	return nil, fmt.Errorf("invalid token")
}

// Middleware returns HTTP middleware for authentication
func (e *AuthorizationEngine) Middleware(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		authHeader := r.Header.Get("Authorization")
		var claims *Claims

		if authHeader != "" {
			parts := strings.Split(authHeader, " ")
			if len(parts) == 2 && strings.ToLower(parts[0]) == "bearer" {
				var err error
				claims, err = e.ValidateToken(parts[1])
				if err != nil {
					claims = &Claims{Role: RoleAnonymous}
				}
			}
		} else if apiKey := r.Header.Get("X-API-Key"); apiKey != "" {
			claims = e.validateAPIKey(r.Context(), apiKey)
		} else {
			claims = &Claims{Role: RoleAnonymous}
		}

		ctx := context.WithValue(r.Context(), "claims", claims)
		r.Header.Set("X-Account-ID", claims.AccountID)
		r.Header.Set("X-Role", string(claims.Role))
		next.ServeHTTP(w, r.WithContext(ctx))
	})
}

func (e *AuthorizationEngine) validateAPIKey(ctx context.Context, apiKey string) *Claims {
	isLive := !strings.HasPrefix(apiKey, "tk_")
	keyColumn := "live_secret_key"
	if !isLive {
		keyColumn = "test_secret_key"
	}

	var accountID string
	err := e.db.QueryRow(ctx, fmt.Sprintf(
		"SELECT id FROM accounts WHERE %s = $1", keyColumn,
	), apiKey).Scan(&accountID)

	if err != nil {
		return &Claims{Role: RoleAnonymous}
	}
	return &Claims{AccountID: accountID, Role: RoleUser, IsLive: isLive}
}

// ApplyRLS modifies a query to add row-level security filters
func (e *AuthorizationEngine) ApplyRLS(query, table string, op Permission, claims *Claims) (string, []interface{}, error) {
	perm := e.GetPermission(table, claims.Role)
	if perm == nil {
		return "", nil, fmt.Errorf("no permission for %s on %s", claims.Role, table)
	}

	var filter map[string]string
	switch op {
	case PermissionSelect:
		if perm.Select == nil || !perm.Select.Allowed {
			return "", nil, fmt.Errorf("select not allowed")
		}
		filter = perm.Select.Filter
	case PermissionUpdate:
		if perm.Update == nil || !perm.Update.Allowed {
			return "", nil, fmt.Errorf("update not allowed")
		}
		filter = perm.Update.Filter
	case PermissionDelete:
		if perm.Delete == nil || !perm.Delete.Allowed {
			return "", nil, fmt.Errorf("delete not allowed")
		}
		filter = perm.Delete.Filter
	case PermissionInsert:
		if perm.Insert == nil || !perm.Insert.Allowed {
			return "", nil, fmt.Errorf("insert not allowed")
		}
		return query, nil, nil
	}

	if filter == nil {
		return query, nil, nil
	}

	conditions := make([]string, 0)
	args := make([]interface{}, 0)
	for col, val := range filter {
		if strings.HasPrefix(val, "X-") {
			val = claims.AccountID
		}
		conditions = append(conditions, fmt.Sprintf("%s = $%d", col, len(args)+1))
		args = append(args, val)
	}

	if strings.Contains(strings.ToUpper(query), "WHERE") {
		query += " AND " + strings.Join(conditions, " AND ")
	} else {
		query += " WHERE " + strings.Join(conditions, " AND ")
	}
	return query, args, nil
}

// PermissionsHandler returns permissions introspection endpoint
func (e *AuthorizationEngine) PermissionsHandler() http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		claims, _ := r.Context().Value("claims").(*Claims)
		if claims == nil {
			claims = &Claims{Role: RoleAnonymous}
		}
		perms := make(map[string]*TablePermission)
		for table, rolePerms := range e.permissions {
			if p, ok := rolePerms[claims.Role]; ok {
				perms[table] = p
			}
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(map[string]interface{}{
			"role": claims.Role, "permissions": perms,
		})
	}
}
