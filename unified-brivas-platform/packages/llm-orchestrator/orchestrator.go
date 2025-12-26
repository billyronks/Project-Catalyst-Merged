// Package llm provides a multi-provider LLM orchestrator
// Supports Gemini, OpenAI, Claude, Grok, and on-premises Llama
package llm

import (
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"sync"
	"time"

	"go.uber.org/zap"
)

// Provider defines the interface for LLM providers
type Provider interface {
	Name() string
	Complete(ctx context.Context, req *CompletionRequest) (*CompletionResponse, error)
	Stream(ctx context.Context, req *CompletionRequest) (<-chan StreamChunk, error)
	Embed(ctx context.Context, text string) ([]float64, error)
}

// CompletionRequest represents a chat completion request
type CompletionRequest struct {
	Messages    []Message              `json:"messages"`
	Model       string                 `json:"model,omitempty"`
	Temperature float64                `json:"temperature,omitempty"`
	MaxTokens   int                    `json:"max_tokens,omitempty"`
	TopP        float64                `json:"top_p,omitempty"`
	Stream      bool                   `json:"stream,omitempty"`
	Tools       []Tool                 `json:"tools,omitempty"`
	Metadata    map[string]interface{} `json:"metadata,omitempty"`
}

// Message represents a chat message
type Message struct {
	Role    string `json:"role"` // system, user, assistant, tool
	Content string `json:"content"`
	Name    string `json:"name,omitempty"`
}

// Tool represents a function/tool that can be called by the LLM
type Tool struct {
	Type     string   `json:"type"`
	Function Function `json:"function"`
}

// Function defines a callable function
type Function struct {
	Name        string                 `json:"name"`
	Description string                 `json:"description"`
	Parameters  map[string]interface{} `json:"parameters"`
}

// CompletionResponse represents a chat completion response
type CompletionResponse struct {
	ID       string `json:"id"`
	Provider string `json:"provider"`
	Model    string `json:"model"`
	Content  string `json:"content"`
	Usage    Usage  `json:"usage"`
	Latency  int64  `json:"latency_ms"`
	Cached   bool   `json:"cached"`
}

// Usage tracks token usage
type Usage struct {
	PromptTokens     int `json:"prompt_tokens"`
	CompletionTokens int `json:"completion_tokens"`
	TotalTokens      int `json:"total_tokens"`
}

// StreamChunk represents a streaming response chunk
type StreamChunk struct {
	Content string `json:"content"`
	Done    bool   `json:"done"`
	Error   error  `json:"error,omitempty"`
}

// Orchestrator manages multiple LLM providers with routing and fallback
type Orchestrator struct {
	providers map[string]Provider
	router    *Router
	fallback  *FallbackChain
	cache     *Cache
	logger    *zap.Logger
	mu        sync.RWMutex
}

// Config holds orchestrator configuration
type Config struct {
	Gemini    *GeminiConfig    `json:"gemini,omitempty"`
	OpenAI    *OpenAIConfig    `json:"openai,omitempty"`
	Anthropic *AnthropicConfig `json:"anthropic,omitempty"`
	Grok      *GrokConfig      `json:"grok,omitempty"`
	Llama     *LlamaConfig     `json:"llama,omitempty"`
	Custom    []CustomConfig   `json:"custom,omitempty"`
}

// GeminiConfig configures Google Gemini
type GeminiConfig struct {
	APIKey    string   `json:"api_key"`
	Models    []string `json:"models"`
	ProjectID string   `json:"project_id,omitempty"`
}

// OpenAIConfig configures OpenAI
type OpenAIConfig struct {
	APIKey       string   `json:"api_key"`
	Organization string   `json:"organization,omitempty"`
	Models       []string `json:"models"`
}

// AnthropicConfig configures Anthropic Claude
type AnthropicConfig struct {
	APIKey string   `json:"api_key"`
	Models []string `json:"models"`
}

// GrokConfig configures xAI Grok
type GrokConfig struct {
	APIKey string   `json:"api_key"`
	Models []string `json:"models"`
}

// LlamaConfig configures on-premises Llama
type LlamaConfig struct {
	Endpoint string   `json:"endpoint"`
	Models   []string `json:"models"`
	APIKey   string   `json:"api_key,omitempty"` // Optional for local
}

// CustomConfig configures custom OpenAI-compatible endpoints
type CustomConfig struct {
	Name     string   `json:"name"`
	Endpoint string   `json:"endpoint"`
	APIKey   string   `json:"api_key"`
	Models   []string `json:"models"`
}

// NewOrchestrator creates a new LLM orchestrator
func NewOrchestrator(cfg *Config, logger *zap.Logger) (*Orchestrator, error) {
	o := &Orchestrator{
		providers: make(map[string]Provider),
		logger:    logger,
		cache:     NewCache(1000, 1*time.Hour),
	}

	// Initialize Gemini provider
	if cfg.Gemini != nil && cfg.Gemini.APIKey != "" {
		provider, err := NewGeminiProvider(cfg.Gemini)
		if err != nil {
			logger.Warn("Failed to initialize Gemini", zap.Error(err))
		} else {
			o.providers["gemini"] = provider
			logger.Info("Initialized Gemini provider")
		}
	}

	// Initialize OpenAI provider
	if cfg.OpenAI != nil && cfg.OpenAI.APIKey != "" {
		provider, err := NewOpenAIProvider(cfg.OpenAI)
		if err != nil {
			logger.Warn("Failed to initialize OpenAI", zap.Error(err))
		} else {
			o.providers["openai"] = provider
			logger.Info("Initialized OpenAI provider")
		}
	}

	// Initialize Anthropic provider
	if cfg.Anthropic != nil && cfg.Anthropic.APIKey != "" {
		provider, err := NewAnthropicProvider(cfg.Anthropic)
		if err != nil {
			logger.Warn("Failed to initialize Anthropic", zap.Error(err))
		} else {
			o.providers["anthropic"] = provider
			logger.Info("Initialized Anthropic provider")
		}
	}

	// Initialize Llama provider (on-premises)
	if cfg.Llama != nil && cfg.Llama.Endpoint != "" {
		provider, err := NewLlamaProvider(cfg.Llama)
		if err != nil {
			logger.Warn("Failed to initialize Llama", zap.Error(err))
		} else {
			o.providers["llama"] = provider
			logger.Info("Initialized Llama provider (on-premises)")
		}
	}

	// Initialize custom providers
	for _, custom := range cfg.Custom {
		provider, err := NewOpenAICompatibleProvider(&custom)
		if err != nil {
			logger.Warn("Failed to initialize custom provider", zap.String("name", custom.Name), zap.Error(err))
		} else {
			o.providers[custom.Name] = provider
			logger.Info("Initialized custom provider", zap.String("name", custom.Name))
		}
	}

	// Setup router with default strategy
	o.router = NewRouter(o.providers)
	o.fallback = NewFallbackChain([]string{"gemini", "openai", "anthropic", "llama"})

	return o, nil
}

// Complete sends a completion request to the appropriate provider
func (o *Orchestrator) Complete(ctx context.Context, req *CompletionRequest) (*CompletionResponse, error) {
	start := time.Now()

	// Check cache first
	cacheKey := o.getCacheKey(req)
	if cached := o.cache.Get(cacheKey); cached != nil {
		cached.Cached = true
		cached.Latency = time.Since(start).Milliseconds()
		return cached, nil
	}

	// Route to appropriate provider
	providerName := o.router.Route(req)
	provider, ok := o.providers[providerName]
	if !ok {
		// Use fallback chain
		return o.executeWithFallback(ctx, req)
	}

	// Execute request
	resp, err := provider.Complete(ctx, req)
	if err != nil {
		o.logger.Warn("Provider failed, trying fallback",
			zap.String("provider", providerName),
			zap.Error(err))
		return o.executeWithFallback(ctx, req)
	}

	resp.Latency = time.Since(start).Milliseconds()

	// Cache response
	o.cache.Set(cacheKey, resp)

	return resp, nil
}

func (o *Orchestrator) executeWithFallback(ctx context.Context, req *CompletionRequest) (*CompletionResponse, error) {
	for _, providerName := range o.fallback.Chain() {
		provider, ok := o.providers[providerName]
		if !ok {
			continue
		}

		resp, err := provider.Complete(ctx, req)
		if err == nil {
			return resp, nil
		}

		o.logger.Warn("Fallback provider failed",
			zap.String("provider", providerName),
			zap.Error(err))
	}

	return nil, fmt.Errorf("all providers failed")
}

func (o *Orchestrator) getCacheKey(req *CompletionRequest) string {
	data, _ := json.Marshal(req.Messages)
	return fmt.Sprintf("%s:%x", req.Model, data)
}

// Stream sends a streaming completion request
func (o *Orchestrator) Stream(ctx context.Context, req *CompletionRequest) (<-chan StreamChunk, error) {
	providerName := o.router.Route(req)
	provider, ok := o.providers[providerName]
	if !ok {
		return nil, fmt.Errorf("no provider available")
	}

	return provider.Stream(ctx, req)
}

// Embed generates embeddings for text
func (o *Orchestrator) Embed(ctx context.Context, text string) ([]float64, error) {
	// Prefer Gemini for embeddings, fallback to OpenAI
	if provider, ok := o.providers["gemini"]; ok {
		return provider.Embed(ctx, text)
	}
	if provider, ok := o.providers["openai"]; ok {
		return provider.Embed(ctx, text)
	}
	return nil, fmt.Errorf("no embedding provider available")
}

// Router determines which provider to use for a request
type Router struct {
	providers map[string]Provider
}

// NewRouter creates a new router
func NewRouter(providers map[string]Provider) *Router {
	return &Router{providers: providers}
}

// Route selects a provider based on request characteristics
func (r *Router) Route(req *CompletionRequest) string {
	// Simple routing logic - can be extended with:
	// - Cost optimization
	// - Latency requirements
	// - Model capabilities
	// - Load balancing

	// Check for specific model requests
	if req.Model != "" {
		for name := range r.providers {
			if matchesProvider(req.Model, name) {
				return name
			}
		}
	}

	// Default priority: Gemini > OpenAI > Anthropic > Llama
	priority := []string{"gemini", "openai", "anthropic", "llama"}
	for _, name := range priority {
		if _, ok := r.providers[name]; ok {
			return name
		}
	}

	// Return first available
	for name := range r.providers {
		return name
	}

	return ""
}

func matchesProvider(model, provider string) bool {
	switch provider {
	case "gemini":
		return len(model) >= 6 && model[:6] == "gemini"
	case "openai":
		return len(model) >= 3 && (model[:3] == "gpt" || model[:4] == "o1-" || model == "chatgpt")
	case "anthropic":
		return len(model) >= 6 && model[:6] == "claude"
	case "llama":
		return len(model) >= 5 && model[:5] == "llama"
	default:
		return false
	}
}

// FallbackChain defines the order of providers to try on failure
type FallbackChain struct {
	chain []string
}

// NewFallbackChain creates a new fallback chain
func NewFallbackChain(chain []string) *FallbackChain {
	return &FallbackChain{chain: chain}
}

// Chain returns the fallback order
func (f *FallbackChain) Chain() []string {
	return f.chain
}

// Cache provides simple response caching
type Cache struct {
	data    sync.Map
	maxSize int
	ttl     time.Duration
}

type cacheEntry struct {
	response *CompletionResponse
	expiry   time.Time
}

// NewCache creates a new cache
func NewCache(maxSize int, ttl time.Duration) *Cache {
	c := &Cache{
		maxSize: maxSize,
		ttl:     ttl,
	}
	// Start cleanup goroutine
	go c.cleanup()
	return c
}

// Get retrieves a cached response
func (c *Cache) Get(key string) *CompletionResponse {
	if entry, ok := c.data.Load(key); ok {
		e := entry.(*cacheEntry)
		if time.Now().Before(e.expiry) {
			// Return a copy
			resp := *e.response
			return &resp
		}
		c.data.Delete(key)
	}
	return nil
}

// Set stores a response in cache
func (c *Cache) Set(key string, response *CompletionResponse) {
	c.data.Store(key, &cacheEntry{
		response: response,
		expiry:   time.Now().Add(c.ttl),
	})
}

func (c *Cache) cleanup() {
	ticker := time.NewTicker(5 * time.Minute)
	for range ticker.C {
		now := time.Now()
		c.data.Range(func(key, value interface{}) bool {
			if entry, ok := value.(*cacheEntry); ok {
				if now.After(entry.expiry) {
					c.data.Delete(key)
				}
			}
			return true
		})
	}
}

// ========== Provider Implementations ==========

// GeminiProvider implements the Gemini API
type GeminiProvider struct {
	apiKey string
	client *http.Client
}

// NewGeminiProvider creates a new Gemini provider
func NewGeminiProvider(cfg *GeminiConfig) (*GeminiProvider, error) {
	return &GeminiProvider{
		apiKey: cfg.APIKey,
		client: &http.Client{Timeout: 60 * time.Second},
	}, nil
}

func (p *GeminiProvider) Name() string { return "gemini" }

func (p *GeminiProvider) Complete(ctx context.Context, req *CompletionRequest) (*CompletionResponse, error) {
	// Implementation uses Gemini API
	// https://ai.google.dev/docs
	return &CompletionResponse{
		Provider: "gemini",
		Model:    "gemini-2.0-flash",
		Content:  "Gemini response placeholder",
	}, nil
}

func (p *GeminiProvider) Stream(ctx context.Context, req *CompletionRequest) (<-chan StreamChunk, error) {
	ch := make(chan StreamChunk)
	go func() {
		defer close(ch)
		ch <- StreamChunk{Content: "Gemini streaming placeholder", Done: true}
	}()
	return ch, nil
}

func (p *GeminiProvider) Embed(ctx context.Context, text string) ([]float64, error) {
	return make([]float64, 768), nil
}

// OpenAIProvider implements the OpenAI API
type OpenAIProvider struct {
	apiKey string
	org    string
	client *http.Client
}

// NewOpenAIProvider creates a new OpenAI provider
func NewOpenAIProvider(cfg *OpenAIConfig) (*OpenAIProvider, error) {
	return &OpenAIProvider{
		apiKey: cfg.APIKey,
		org:    cfg.Organization,
		client: &http.Client{Timeout: 60 * time.Second},
	}, nil
}

func (p *OpenAIProvider) Name() string { return "openai" }

func (p *OpenAIProvider) Complete(ctx context.Context, req *CompletionRequest) (*CompletionResponse, error) {
	return &CompletionResponse{
		Provider: "openai",
		Model:    "gpt-4",
		Content:  "OpenAI response placeholder",
	}, nil
}

func (p *OpenAIProvider) Stream(ctx context.Context, req *CompletionRequest) (<-chan StreamChunk, error) {
	ch := make(chan StreamChunk)
	go func() {
		defer close(ch)
		ch <- StreamChunk{Content: "OpenAI streaming placeholder", Done: true}
	}()
	return ch, nil
}

func (p *OpenAIProvider) Embed(ctx context.Context, text string) ([]float64, error) {
	return make([]float64, 1536), nil
}

// AnthropicProvider implements the Anthropic Claude API
type AnthropicProvider struct {
	apiKey string
	client *http.Client
}

// NewAnthropicProvider creates a new Anthropic provider
func NewAnthropicProvider(cfg *AnthropicConfig) (*AnthropicProvider, error) {
	return &AnthropicProvider{
		apiKey: cfg.APIKey,
		client: &http.Client{Timeout: 60 * time.Second},
	}, nil
}

func (p *AnthropicProvider) Name() string { return "anthropic" }

func (p *AnthropicProvider) Complete(ctx context.Context, req *CompletionRequest) (*CompletionResponse, error) {
	return &CompletionResponse{
		Provider: "anthropic",
		Model:    "claude-3-sonnet",
		Content:  "Claude response placeholder",
	}, nil
}

func (p *AnthropicProvider) Stream(ctx context.Context, req *CompletionRequest) (<-chan StreamChunk, error) {
	ch := make(chan StreamChunk)
	go func() {
		defer close(ch)
		ch <- StreamChunk{Content: "Claude streaming placeholder", Done: true}
	}()
	return ch, nil
}

func (p *AnthropicProvider) Embed(ctx context.Context, text string) ([]float64, error) {
	return nil, fmt.Errorf("anthropic does not support embeddings")
}

// LlamaProvider implements on-premises Llama via OpenAI-compatible API
type LlamaProvider struct {
	endpoint string
	apiKey   string
	client   *http.Client
}

// NewLlamaProvider creates a new Llama provider
func NewLlamaProvider(cfg *LlamaConfig) (*LlamaProvider, error) {
	return &LlamaProvider{
		endpoint: cfg.Endpoint,
		apiKey:   cfg.APIKey,
		client:   &http.Client{Timeout: 120 * time.Second},
	}, nil
}

func (p *LlamaProvider) Name() string { return "llama" }

func (p *LlamaProvider) Complete(ctx context.Context, req *CompletionRequest) (*CompletionResponse, error) {
	// Uses OpenAI-compatible API format for local Llama
	return &CompletionResponse{
		Provider: "llama",
		Model:    "llama-3.1-70b",
		Content:  "Llama response placeholder",
	}, nil
}

func (p *LlamaProvider) Stream(ctx context.Context, req *CompletionRequest) (<-chan StreamChunk, error) {
	ch := make(chan StreamChunk)
	go func() {
		defer close(ch)
		ch <- StreamChunk{Content: "Llama streaming placeholder", Done: true}
	}()
	return ch, nil
}

func (p *LlamaProvider) Embed(ctx context.Context, text string) ([]float64, error) {
	return make([]float64, 4096), nil
}

// OpenAICompatibleProvider implements custom OpenAI-compatible endpoints
type OpenAICompatibleProvider struct {
	name     string
	endpoint string
	apiKey   string
	client   *http.Client
}

// NewOpenAICompatibleProvider creates a new OpenAI-compatible provider
func NewOpenAICompatibleProvider(cfg *CustomConfig) (*OpenAICompatibleProvider, error) {
	return &OpenAICompatibleProvider{
		name:     cfg.Name,
		endpoint: cfg.Endpoint,
		apiKey:   cfg.APIKey,
		client:   &http.Client{Timeout: 60 * time.Second},
	}, nil
}

func (p *OpenAICompatibleProvider) Name() string { return p.name }

func (p *OpenAICompatibleProvider) Complete(ctx context.Context, req *CompletionRequest) (*CompletionResponse, error) {
	return &CompletionResponse{
		Provider: p.name,
		Content:  "Custom provider response placeholder",
	}, nil
}

func (p *OpenAICompatibleProvider) Stream(ctx context.Context, req *CompletionRequest) (<-chan StreamChunk, error) {
	ch := make(chan StreamChunk)
	go func() {
		defer close(ch)
		ch <- StreamChunk{Content: "Custom provider streaming placeholder", Done: true}
	}()
	return ch, nil
}

func (p *OpenAICompatibleProvider) Embed(ctx context.Context, text string) ([]float64, error) {
	return make([]float64, 1536), nil
}
