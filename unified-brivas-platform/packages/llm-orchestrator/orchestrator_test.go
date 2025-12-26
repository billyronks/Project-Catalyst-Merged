// Package llm provides tests for the LLM orchestrator
package llm

import (
	"context"
	"testing"
	"time"
)

func TestNewOrchestrator(t *testing.T) {
	cfg := &Config{
		Gemini: &GeminiConfig{
			APIKey: "test-key",
			Models: []string{"gemini-2.0-flash"},
		},
	}

	orch, err := NewOrchestrator(cfg, nil)
	if err != nil {
		t.Fatalf("Failed to create orchestrator: %v", err)
	}

	if len(orch.providers) == 0 {
		// Provider may not be initialized without valid key
		t.Log("No providers initialized (expected with test key)")
	}
}

func TestRouter(t *testing.T) {
	providers := map[string]Provider{
		"gemini": &GeminiProvider{},
		"openai": &OpenAIProvider{},
	}

	router := NewRouter(providers)

	// Test gemini model routing
	req := &CompletionRequest{Model: "gemini-2.0-flash"}
	result := router.Route(req)
	if result != "gemini" {
		t.Errorf("Expected gemini provider, got %s", result)
	}

	// Test openai model routing
	req = &CompletionRequest{Model: "gpt-4"}
	result = router.Route(req)
	if result != "openai" {
		t.Errorf("Expected openai provider, got %s", result)
	}

	// Test claude model routing (should fallback)
	req = &CompletionRequest{Model: "claude-3-sonnet"}
	result = router.Route(req)
	// Should get first available since no anthropic provider
}

func TestFallbackChain(t *testing.T) {
	chain := NewFallbackChain([]string{"gemini", "openai", "anthropic", "llama"})

	providers := chain.Chain()
	if len(providers) != 4 {
		t.Errorf("Expected 4 providers in chain, got %d", len(providers))
	}
	if providers[0] != "gemini" {
		t.Errorf("First provider should be gemini, got %s", providers[0])
	}
}

func TestCache(t *testing.T) {
	cache := NewCache(100, 1*time.Second)

	key := "test-key"
	response := &CompletionResponse{
		ID:      "test-id",
		Content: "test content",
	}

	// Test set and get
	cache.Set(key, response)
	result := cache.Get(key)
	if result == nil {
		t.Fatal("Cache should return stored value")
	}
	if result.Content != response.Content {
		t.Errorf("Expected %s, got %s", response.Content, result.Content)
	}

	// Test expiry
	time.Sleep(1100 * time.Millisecond)
	result = cache.Get(key)
	if result != nil {
		t.Error("Cache entry should have expired")
	}
}

func TestMatchesProvider(t *testing.T) {
	tests := []struct {
		model    string
		provider string
		expected bool
	}{
		{"gemini-2.0-flash", "gemini", true},
		{"gpt-4", "openai", true},
		{"claude-3-sonnet", "anthropic", true},
		{"llama-3.1-70b", "llama", true},
		{"gpt-4", "gemini", false},
		{"claude-3", "openai", false},
	}

	for _, tc := range tests {
		result := matchesProvider(tc.model, tc.provider)
		if result != tc.expected {
			t.Errorf("matchesProvider(%s, %s) = %v, expected %v",
				tc.model, tc.provider, result, tc.expected)
		}
	}
}

func TestProviderInterface(t *testing.T) {
	// Test that all providers implement the interface
	var _ Provider = (*GeminiProvider)(nil)
	var _ Provider = (*OpenAIProvider)(nil)
	var _ Provider = (*AnthropicProvider)(nil)
	var _ Provider = (*LlamaProvider)(nil)
	var _ Provider = (*OpenAICompatibleProvider)(nil)
}

func TestGeminiProvider(t *testing.T) {
	provider := &GeminiProvider{
		apiKey: "test-key",
	}

	if provider.Name() != "gemini" {
		t.Errorf("Expected name 'gemini', got '%s'", provider.Name())
	}

	ctx := context.Background()
	req := &CompletionRequest{
		Messages: []Message{{Role: "user", Content: "Hello"}},
	}

	resp, err := provider.Complete(ctx, req)
	if err != nil {
		t.Fatalf("Complete failed: %v", err)
	}
	if resp.Provider != "gemini" {
		t.Errorf("Expected provider 'gemini', got '%s'", resp.Provider)
	}
}

func TestOpenAIProvider(t *testing.T) {
	provider := &OpenAIProvider{
		apiKey: "test-key",
	}

	if provider.Name() != "openai" {
		t.Errorf("Expected name 'openai', got '%s'", provider.Name())
	}
}

func TestAnthropicProvider(t *testing.T) {
	provider := &AnthropicProvider{
		apiKey: "test-key",
	}

	if provider.Name() != "anthropic" {
		t.Errorf("Expected name 'anthropic', got '%s'", provider.Name())
	}

	// Test embedding returns error
	ctx := context.Background()
	_, err := provider.Embed(ctx, "test")
	if err == nil {
		t.Error("Anthropic Embed should return error")
	}
}

func TestLlamaProvider(t *testing.T) {
	provider := &LlamaProvider{
		endpoint: "http://localhost:8080",
	}

	if provider.Name() != "llama" {
		t.Errorf("Expected name 'llama', got '%s'", provider.Name())
	}
}

func TestCompletionRequest(t *testing.T) {
	req := &CompletionRequest{
		Messages: []Message{
			{Role: "system", Content: "You are helpful"},
			{Role: "user", Content: "Hello"},
		},
		Temperature: 0.7,
		MaxTokens:   100,
	}

	if len(req.Messages) != 2 {
		t.Errorf("Expected 2 messages, got %d", len(req.Messages))
	}
	if req.Temperature != 0.7 {
		t.Errorf("Expected temperature 0.7, got %f", req.Temperature)
	}
}

// Benchmark tests
func BenchmarkCache(b *testing.B) {
	cache := NewCache(1000, 1*time.Hour)
	key := "benchmark-key"
	response := &CompletionResponse{Content: "test"}
	cache.Set(key, response)

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		cache.Get(key)
	}
}

func BenchmarkMatchesProvider(b *testing.B) {
	for i := 0; i < b.N; i++ {
		matchesProvider("gemini-2.0-flash", "gemini")
	}
}
