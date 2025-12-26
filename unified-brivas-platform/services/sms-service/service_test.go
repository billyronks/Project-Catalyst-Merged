// Package sms provides tests for the SMS service
package sms

import (
	"bytes"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
)

// MockDB implements a mock database for testing
type MockDB struct {
	accounts   map[string]map[string]interface{}
	smsHistory []map[string]interface{}
	senderIDs  map[string]bool
}

func NewMockDB() *MockDB {
	return &MockDB{
		accounts: map[string]map[string]interface{}{
			"BV123456789": {
				"id":      "BV123456789",
				"balance": 1000.0,
				"email":   "test@example.com",
			},
		},
		smsHistory: make([]map[string]interface{}, 0),
		senderIDs: map[string]bool{
			"BRIVAS": true,
		},
	}
}

func TestFormatNumber(t *testing.T) {
	tests := []struct {
		input    string
		expected string
	}{
		{"+2348012345678", "2348012345678"},
		{"08012345678", "2348012345678"},
		{"2348012345678", "2348012345678"},
		{" 08012345678 ", "2348012345678"},
	}

	svc := &Service{
		networkCodes: map[string]string{
			"0801": "MTN",
		},
	}

	for _, tc := range tests {
		result := svc.formatNumber(tc.input)
		if result != tc.expected {
			t.Errorf("formatNumber(%s) = %s, expected %s", tc.input, result, tc.expected)
		}
	}
}

func TestGetNetwork(t *testing.T) {
	svc := &Service{
		networkCodes: map[string]string{
			"0803": "MTN", "0806": "MTN", "0703": "MTN",
			"0805": "GLO", "0807": "GLO",
			"0802": "AIRTEL", "0808": "AIRTEL",
			"0809": "9MOBILE", "0817": "9MOBILE",
		},
	}

	tests := []struct {
		number   string
		expected string
	}{
		{"2348031234567", "MTN"},
		{"08061234567", "MTN"},
		{"2348051234567", "GLO"},
		{"08021234567", "AIRTEL"},
		{"2348091234567", "9MOBILE"},
	}

	for _, tc := range tests {
		result := svc.getNetwork(tc.number)
		if result != tc.expected {
			t.Errorf("getNetwork(%s) = %s, expected %s", tc.number, result, tc.expected)
		}
	}
}

func TestGetRate(t *testing.T) {
	svc := &Service{}

	tests := []struct {
		msgType  string
		expected float64
	}{
		{"otp", 3.0},
		{"transactional", 3.0},
		{"promotional", 2.5},
		{"unknown", 3.0},
	}

	for _, tc := range tests {
		result := svc.getRate(tc.msgType, "")
		if result != tc.expected {
			t.Errorf("getRate(%s) = %f, expected %f", tc.msgType, result, tc.expected)
		}
	}
}

func TestGenerateSID(t *testing.T) {
	svc := &Service{}

	sid1 := svc.generateSID("BV123456789", "BULK")
	sid2 := svc.generateSID("BV123456789", "BULK")

	if sid1 == sid2 {
		t.Error("generateSID should produce unique IDs")
	}

	if len(sid1) < 10 {
		t.Error("SID should be reasonably long")
	}
}

func TestDLRBuffer(t *testing.T) {
	buffer := &DLRBuffer{
		delivered: make([]string, 0),
		failed:    make([]string, 0),
	}

	// Test queuing
	buffer.queue("msg1", "delivered")
	buffer.queue("msg2", "delivered")
	buffer.queue("msg3", "failed")

	if len(buffer.delivered) != 2 {
		t.Errorf("Expected 2 delivered, got %d", len(buffer.delivered))
	}
	if len(buffer.failed) != 1 {
		t.Errorf("Expected 1 failed, got %d", len(buffer.failed))
	}
}

func TestHandleSendValidation(t *testing.T) {
	svc := &Service{
		networkCodes: map[string]string{},
	}
	handler := http.HandlerFunc(svc.handleSend)

	// Test missing fields
	body := bytes.NewBufferString(`{}`)
	req := httptest.NewRequest("POST", "/send", body)
	req.Header.Set("Content-Type", "application/json")

	rr := httptest.NewRecorder()
	handler.ServeHTTP(rr, req)

	if rr.Code != http.StatusBadRequest {
		t.Errorf("Expected status 400, got %d", rr.Code)
	}
}

func TestHandleBulkSendValidation(t *testing.T) {
	svc := &Service{
		networkCodes: map[string]string{},
	}
	handler := http.HandlerFunc(svc.handleBulkSend)

	// Test missing fields
	body := bytes.NewBufferString(`{"to": []}`)
	req := httptest.NewRequest("POST", "/bulk", body)
	req.Header.Set("Content-Type", "application/json")

	rr := httptest.NewRecorder()
	handler.ServeHTTP(rr, req)

	if rr.Code != http.StatusBadRequest {
		t.Errorf("Expected status 400, got %d", rr.Code)
	}
}

func TestHandleBulkSendTooManyRecipients(t *testing.T) {
	svc := &Service{
		networkCodes: map[string]string{},
	}
	handler := http.HandlerFunc(svc.handleBulkSend)

	// Create request with too many recipients
	recipients := make([]string, 1001)
	for i := range recipients {
		recipients[i] = "08012345678"
	}
	reqBody := map[string]interface{}{
		"to":      recipients,
		"message": "test",
		"type":    "promotional",
	}
	body, _ := json.Marshal(reqBody)

	req := httptest.NewRequest("POST", "/bulk", bytes.NewBuffer(body))
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("X-Is-Live", "true")

	rr := httptest.NewRecorder()
	handler.ServeHTTP(rr, req)

	if rr.Code != http.StatusBadRequest {
		t.Errorf("Expected status 400 for too many recipients, got %d", rr.Code)
	}
}

// Integration test example
func TestSMSServiceIntegration(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping integration test in short mode")
	}

	// This would test with actual LumaDB connection
	// Requires running LumaDB instance
	t.Log("Integration test would run here")
}

// Benchmark tests
func BenchmarkFormatNumber(b *testing.B) {
	svc := &Service{
		networkCodes: map[string]string{},
	}

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		svc.formatNumber("+2348012345678")
	}
}

func BenchmarkGetNetwork(b *testing.B) {
	svc := &Service{
		networkCodes: map[string]string{
			"0803": "MTN", "0806": "MTN", "0703": "MTN",
			"0805": "GLO", "0807": "GLO",
			"0802": "AIRTEL", "0808": "AIRTEL",
			"0809": "9MOBILE", "0817": "9MOBILE",
		},
	}

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		svc.getNetwork("2348031234567")
	}
}

func BenchmarkGenerateSID(b *testing.B) {
	svc := &Service{}

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		svc.generateSID("BV123456789", "BULK")
	}
}
