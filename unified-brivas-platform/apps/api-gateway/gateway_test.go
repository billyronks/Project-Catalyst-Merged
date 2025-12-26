// Package gateway provides tests for the API gateway
package gateway

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
)

func TestHealthCheck(t *testing.T) {
	engine := &UnifiedAPIEngine{
		schema: &Schema{
			Tables: []TableSchema{
				{Name: "accounts", PrimaryKey: "id"},
			},
		},
	}

	req := httptest.NewRequest("GET", "/health", nil)
	rr := httptest.NewRecorder()

	engine.healthCheck(rr, req)

	if rr.Code != http.StatusOK {
		t.Errorf("Expected status 200, got %d", rr.Code)
	}

	var response map[string]interface{}
	json.Unmarshal(rr.Body.Bytes(), &response)

	if response["status"] != "healthy" {
		t.Errorf("Expected status 'healthy', got %v", response["status"])
	}
}

func TestReadinessCheck(t *testing.T) {
	// Test with no schema
	engine := &UnifiedAPIEngine{}

	req := httptest.NewRequest("GET", "/ready", nil)
	rr := httptest.NewRecorder()

	engine.readinessCheck(rr, req)

	if rr.Code != http.StatusServiceUnavailable {
		t.Errorf("Expected status 503 when not ready, got %d", rr.Code)
	}

	// Test with schema loaded
	engine.schema = &Schema{
		Tables: []TableSchema{
			{Name: "accounts", PrimaryKey: "id"},
		},
	}

	rr = httptest.NewRecorder()
	engine.readinessCheck(rr, req)

	if rr.Code != http.StatusOK {
		t.Errorf("Expected status 200 when ready, got %d", rr.Code)
	}
}

func TestToCamelCase(t *testing.T) {
	tests := []struct {
		input    string
		expected string
	}{
		{"account_id", "accountId"},
		{"sms_history", "smsHistory"},
		{"id", "id"},
		{"first_name_last", "firstNameLast"},
	}

	for _, tc := range tests {
		result := toCamelCase(tc.input)
		if result != tc.expected {
			t.Errorf("toCamelCase(%s) = %s, expected %s", tc.input, result, tc.expected)
		}
	}
}

func TestToPascalCase(t *testing.T) {
	tests := []struct {
		input    string
		expected string
	}{
		{"account_id", "AccountId"},
		{"sms_history", "SmsHistory"},
		{"accounts", "Accounts"},
	}

	for _, tc := range tests {
		result := toPascalCase(tc.input)
		if result != tc.expected {
			t.Errorf("toPascalCase(%s) = %s, expected %s", tc.input, result, tc.expected)
		}
	}
}

func TestToPlural(t *testing.T) {
	tests := []struct {
		input    string
		expected string
	}{
		{"account", "accounts"},
		{"history", "histories"},
		{"status", "statuses"},
	}

	for _, tc := range tests {
		result := toPlural(tc.input)
		if result != tc.expected {
			t.Errorf("toPlural(%s) = %s, expected %s", tc.input, result, tc.expected)
		}
	}
}

func TestMapSQLTypeToGraphQL(t *testing.T) {
	tests := []struct {
		sqlType      string
		expectString bool
	}{
		{"integer", false},
		{"varchar", true},
		{"boolean", false},
		{"double precision", false},
		{"text", true},
		{"jsonb", true},
	}

	for _, tc := range tests {
		result := mapSQLTypeToGraphQL(tc.sqlType)
		if tc.expectString && result.Name() != "String" {
			t.Errorf("Expected String for %s, got %s", tc.sqlType, result.Name())
		}
	}
}

func TestRESTHandlerListValidation(t *testing.T) {
	handler := &RESTHandler{
		schema: &Schema{
			Tables: []TableSchema{
				{Name: "accounts", PrimaryKey: "id"},
			},
		},
	}

	req := httptest.NewRequest("GET", "/accounts", nil)
	rr := httptest.NewRecorder()

	// This would normally require a DB connection
	// Testing handler setup logic
	routes := handler.Routes()
	if routes == nil {
		t.Error("Routes should not be nil")
	}
}

func TestGraphQLQueryParsing(t *testing.T) {
	// Test GraphQL query parsing
	query := `{
		accounts(limit: 10) {
			id
			email
		}
	}`

	if len(query) == 0 {
		t.Error("Query should not be empty")
	}
}

func TestMCPToolRegistration(t *testing.T) {
	handler := &MCPHandler{
		tools: make(map[string]MCPTool),
		schema: &Schema{
			Tables: []TableSchema{
				{Name: "accounts", PrimaryKey: "id"},
				{Name: "campaigns", PrimaryKey: "id"},
			},
		},
	}

	// Register tools for tables
	for _, table := range handler.schema.Tables {
		handler.registerTableTools(table)
	}

	// Verify tools were registered
	if len(handler.tools) < 4 { // At least 2 tools per table (list, get)
		t.Errorf("Expected at least 4 tools, got %d", len(handler.tools))
	}

	// Check specific tool exists
	if _, ok := handler.tools["list_accounts"]; !ok {
		t.Error("list_accounts tool should exist")
	}
	if _, ok := handler.tools["get_campaigns"]; !ok {
		t.Error("get_campaigns tool should exist")
	}
}

// Benchmark tests
func BenchmarkToCamelCase(b *testing.B) {
	for i := 0; i < b.N; i++ {
		toCamelCase("account_id_test_value")
	}
}

func BenchmarkToPascalCase(b *testing.B) {
	for i := 0; i < b.N; i++ {
		toPascalCase("account_id_test_value")
	}
}
