// Package ai provides AI-powered platform features using LLM integration
package ai

import (
	"encoding/json"
	"fmt"
	"net/http"
	"strings"
	"time"

	"github.com/go-chi/chi/v5"
	"go.uber.org/zap"

	llm "github.com/brivas/unified-platform/packages/llm-orchestrator"
	lumadb "github.com/brivas/unified-platform/packages/lumadb-client"
)

// Service provides AI-powered platform features
type Service struct {
	db     *lumadb.Client
	llm    *llm.Orchestrator
	logger *zap.Logger
}

// NewService creates a new AI service
func NewService(db *lumadb.Client, llmOrch *llm.Orchestrator, logger *zap.Logger) *Service {
	return &Service{db: db, llm: llmOrch, logger: logger}
}

// Routes returns Chi router with AI endpoints
func (s *Service) Routes() chi.Router {
	r := chi.NewRouter()

	// SMS Content Generation
	r.Post("/sms/generate", s.handleGenerateSMS)
	r.Post("/sms/improve", s.handleImproveSMS)
	r.Post("/sms/translate", s.handleTranslateSMS)

	// Campaign Optimization
	r.Post("/campaign/optimize", s.handleOptimizeCampaign)
	r.Post("/campaign/schedule", s.handleOptimalSchedule)
	r.Post("/campaign/segment", s.handleAudienceSegmentation)

	// Fraud Detection
	r.Post("/fraud/analyze", s.handleFraudAnalysis)
	r.Post("/fraud/score", s.handleFraudScore)

	// Customer Support
	r.Post("/support/respond", s.handleSupportResponse)
	r.Post("/support/categorize", s.handleCategorizeTicket)

	// Analytics & Insights
	r.Post("/analytics/summarize", s.handleSummarize)
	r.Get("/analytics/insights/{account_id}", s.handleAccountInsights)

	// Chat Interface
	r.Post("/chat", s.handleChat)

	return r
}

// ============== SMS Content Generation ==============

type GenerateSMSRequest struct {
	Purpose    string   `json:"purpose"` // promotional, transactional, notification
	Product    string   `json:"product"`
	Audience   string   `json:"audience"`
	Tone       string   `json:"tone"` // formal, friendly, urgent
	Keywords   []string `json:"keywords"`
	MaxLength  int      `json:"max_length"`
	Variations int      `json:"variations"`
}

func (s *Service) handleGenerateSMS(w http.ResponseWriter, r *http.Request) {
	ctx := r.Context()
	var req GenerateSMSRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		s.jsonError(w, "invalid request", http.StatusBadRequest)
		return
	}

	if req.MaxLength == 0 {
		req.MaxLength = 160
	}
	if req.Variations == 0 {
		req.Variations = 3
	}

	prompt := fmt.Sprintf(`Generate %d SMS message variations for:
Purpose: %s
Product/Service: %s
Target Audience: %s
Tone: %s
Keywords to include: %s
Maximum length: %d characters

Return as JSON array with "content" and "char_count" for each variation.
Ensure messages are engaging, clear, and include a call-to-action.`,
		req.Variations, req.Purpose, req.Product, req.Audience, req.Tone,
		strings.Join(req.Keywords, ", "), req.MaxLength)

	resp, err := s.llm.Complete(ctx, &llm.CompletionRequest{
		Messages:    []llm.Message{{Role: "user", Content: prompt}},
		Temperature: 0.8,
	})
	if err != nil {
		s.jsonError(w, "AI generation failed", http.StatusInternalServerError)
		return
	}

	s.jsonResponse(w, map[string]interface{}{
		"status":   "success",
		"messages": json.RawMessage(resp.Content),
		"model":    resp.Model,
	}, http.StatusOK)
}

func (s *Service) handleImproveSMS(w http.ResponseWriter, r *http.Request) {
	ctx := r.Context()
	var req struct {
		Content string `json:"content"`
		Goal    string `json:"goal"` // engagement, clarity, urgency
	}
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		s.jsonError(w, "invalid request", http.StatusBadRequest)
		return
	}

	prompt := fmt.Sprintf(`Improve this SMS message for better %s:
Original: "%s"

Provide:
1. Improved version (max 160 chars)
2. Explanation of changes
3. Predicted engagement score (1-10)

Return as JSON with "improved", "explanation", "score" fields.`, req.Goal, req.Content)

	resp, err := s.llm.Complete(ctx, &llm.CompletionRequest{
		Messages: []llm.Message{{Role: "user", Content: prompt}},
	})
	if err != nil {
		s.jsonError(w, "AI improvement failed", http.StatusInternalServerError)
		return
	}

	s.jsonResponse(w, map[string]interface{}{
		"status": "success",
		"result": json.RawMessage(resp.Content),
	}, http.StatusOK)
}

func (s *Service) handleTranslateSMS(w http.ResponseWriter, r *http.Request) {
	ctx := r.Context()
	var req struct {
		Content   string   `json:"content"`
		Languages []string `json:"languages"` // Target languages
	}
	json.NewDecoder(r.Body).Decode(&req)

	prompt := fmt.Sprintf(`Translate this SMS to %s while maintaining the tone and staying under 160 chars:
"%s"

Return JSON object with language codes as keys and translations as values.`,
		strings.Join(req.Languages, ", "), req.Content)

	resp, err := s.llm.Complete(ctx, &llm.CompletionRequest{
		Messages: []llm.Message{{Role: "user", Content: prompt}},
	})
	if err != nil {
		s.jsonError(w, "translation failed", http.StatusInternalServerError)
		return
	}

	s.jsonResponse(w, map[string]interface{}{
		"status":       "success",
		"translations": json.RawMessage(resp.Content),
	}, http.StatusOK)
}

// ============== Campaign Optimization ==============

func (s *Service) handleOptimizeCampaign(w http.ResponseWriter, r *http.Request) {
	ctx := r.Context()
	var req struct {
		CampaignID   string                 `json:"campaign_id"`
		CurrentStats map[string]interface{} `json:"current_stats"`
	}
	json.NewDecoder(r.Body).Decode(&req)

	// Get historical campaign data
	statsJSON, _ := json.Marshal(req.CurrentStats)

	prompt := fmt.Sprintf(`Analyze this SMS campaign and provide optimization recommendations:
Campaign Stats: %s

Provide:
1. Performance assessment
2. Top 3 improvement recommendations
3. Suggested A/B test variations
4. Predicted improvement percentage

Return as structured JSON.`, string(statsJSON))

	resp, err := s.llm.Complete(ctx, &llm.CompletionRequest{
		Messages: []llm.Message{{Role: "user", Content: prompt}},
	})
	if err != nil {
		s.jsonError(w, "optimization failed", http.StatusInternalServerError)
		return
	}

	s.jsonResponse(w, map[string]interface{}{
		"status":          "success",
		"recommendations": json.RawMessage(resp.Content),
	}, http.StatusOK)
}

func (s *Service) handleOptimalSchedule(w http.ResponseWriter, r *http.Request) {
	ctx := r.Context()
	var req struct {
		AccountID string `json:"account_id"`
		Audience  string `json:"audience"`
		Timezone  string `json:"timezone"`
		DaysAhead int    `json:"days_ahead"`
	}
	json.NewDecoder(r.Body).Decode(&req)

	// Query historical delivery data
	var peakHours []map[string]interface{}
	rows, _ := s.db.Query(ctx, `
		SELECT EXTRACT(HOUR FROM sent_time) as hour, 
			   COUNT(*) as total,
			   SUM(CASE WHEN status = 'delivered' THEN 1 ELSE 0 END) as delivered
		FROM sms_history 
		WHERE account_id = $1
		GROUP BY hour
		ORDER BY delivered DESC
		LIMIT 5
	`, req.AccountID)
	defer rows.Close()
	for rows.Next() {
		var hour, total, delivered int
		rows.Scan(&hour, &total, &delivered)
		peakHours = append(peakHours, map[string]interface{}{
			"hour": hour, "total": total, "delivered": delivered,
		})
	}

	prompt := fmt.Sprintf(`Based on these delivery patterns, recommend optimal send times:
Historical peak hours: %v
Target audience: %s
Timezone: %s
Plan for next %d days

Return JSON with recommended schedule slots.`, peakHours, req.Audience, req.Timezone, req.DaysAhead)

	resp, err := s.llm.Complete(ctx, &llm.CompletionRequest{
		Messages: []llm.Message{{Role: "user", Content: prompt}},
	})
	if err != nil {
		s.jsonError(w, "scheduling failed", http.StatusInternalServerError)
		return
	}

	s.jsonResponse(w, map[string]interface{}{
		"status":           "success",
		"schedule":         json.RawMessage(resp.Content),
		"historical_peaks": peakHours,
	}, http.StatusOK)
}

func (s *Service) handleAudienceSegmentation(w http.ResponseWriter, r *http.Request) {
	ctx := r.Context()
	var req struct {
		AccountID string `json:"account_id"`
		Criteria  string `json:"criteria"`
	}
	json.NewDecoder(r.Body).Decode(&req)

	// Get account's contact data characteristics
	prompt := fmt.Sprintf(`Create audience segments for SMS marketing based on: %s

Suggest 4-6 segments with:
- Segment name
- Description
- Recommended message tone
- Best send times
- Expected engagement rate

Return as JSON array.`, req.Criteria)

	resp, err := s.llm.Complete(ctx, &llm.CompletionRequest{
		Messages: []llm.Message{{Role: "user", Content: prompt}},
	})
	if err != nil {
		s.jsonError(w, "segmentation failed", http.StatusInternalServerError)
		return
	}

	s.jsonResponse(w, map[string]interface{}{
		"status":   "success",
		"segments": json.RawMessage(resp.Content),
	}, http.StatusOK)
}

// ============== Fraud Detection ==============

func (s *Service) handleFraudAnalysis(w http.ResponseWriter, r *http.Request) {
	ctx := r.Context()
	var req struct {
		AccountID string `json:"account_id"`
		TimeRange string `json:"time_range"` // 24h, 7d, 30d
	}
	json.NewDecoder(r.Body).Decode(&req)

	// Get recent activity patterns
	var patterns []map[string]interface{}
	rows, _ := s.db.Query(ctx, `
		SELECT sender, recipient, COUNT(*) as count,
			   COUNT(DISTINCT recipient) as unique_recipients
		FROM sms_history 
		WHERE account_id = $1 AND sent_date >= CURRENT_DATE - INTERVAL '7 days'
		GROUP BY sender, recipient
		HAVING COUNT(*) > 10
		ORDER BY count DESC
		LIMIT 20
	`, req.AccountID)
	defer rows.Close()
	for rows.Next() {
		var sender, recipient string
		var count, unique int
		rows.Scan(&sender, &recipient, &count, &unique)
		patterns = append(patterns, map[string]interface{}{
			"sender": sender, "recipient": recipient,
			"count": count, "unique_recipients": unique,
		})
	}

	prompt := fmt.Sprintf(`Analyze these SMS patterns for potential fraud indicators:
%v

Identify:
1. Suspicious patterns (spam, phishing, fraud)
2. Risk score (0-100)
3. Specific concerns
4. Recommended actions

Return structured JSON.`, patterns)

	resp, err := s.llm.Complete(ctx, &llm.CompletionRequest{
		Messages: []llm.Message{{Role: "user", Content: prompt}},
	})
	if err != nil {
		s.jsonError(w, "analysis failed", http.StatusInternalServerError)
		return
	}

	s.jsonResponse(w, map[string]interface{}{
		"status":            "success",
		"analysis":          json.RawMessage(resp.Content),
		"patterns_analyzed": len(patterns),
	}, http.StatusOK)
}

func (s *Service) handleFraudScore(w http.ResponseWriter, r *http.Request) {
	ctx := r.Context()
	var req struct {
		Message string `json:"message"`
		Sender  string `json:"sender"`
		Volume  int    `json:"volume"`
	}
	json.NewDecoder(r.Body).Decode(&req)

	prompt := fmt.Sprintf(`Score this SMS for fraud risk (0-100):
Message: "%s"
Sender ID: %s
Send volume: %d

Evaluate:
- Phishing indicators
- Spam characteristics
- Impersonation attempts
- Malicious links

Return JSON with "score", "risk_level", "indicators".`, req.Message, req.Sender, req.Volume)

	resp, err := s.llm.Complete(ctx, &llm.CompletionRequest{
		Messages: []llm.Message{{Role: "user", Content: prompt}},
	})
	if err != nil {
		s.jsonError(w, "scoring failed", http.StatusInternalServerError)
		return
	}

	s.jsonResponse(w, map[string]interface{}{
		"status": "success",
		"result": json.RawMessage(resp.Content),
	}, http.StatusOK)
}

// ============== Customer Support ==============

func (s *Service) handleSupportResponse(w http.ResponseWriter, r *http.Request) {
	ctx := r.Context()
	var req struct {
		Query    string `json:"query"`
		Context  string `json:"context"`
		Language string `json:"language"`
	}
	json.NewDecoder(r.Body).Decode(&req)

	systemPrompt := `You are a helpful customer support agent for Brivas, an SMS and telecommunications platform.
Answer questions about: SMS sending, billing, sender IDs, campaigns, API usage, and account management.
Be concise, professional, and helpful. If you don't know something, suggest contacting support.`

	resp, err := s.llm.Complete(ctx, &llm.CompletionRequest{
		Messages: []llm.Message{
			{Role: "system", Content: systemPrompt},
			{Role: "user", Content: req.Query},
		},
	})
	if err != nil {
		s.jsonError(w, "response generation failed", http.StatusInternalServerError)
		return
	}

	s.jsonResponse(w, map[string]interface{}{
		"status":   "success",
		"response": resp.Content,
		"model":    resp.Model,
	}, http.StatusOK)
}

func (s *Service) handleCategorizeTicket(w http.ResponseWriter, r *http.Request) {
	ctx := r.Context()
	var req struct {
		Subject string `json:"subject"`
		Body    string `json:"body"`
	}
	json.NewDecoder(r.Body).Decode(&req)

	prompt := fmt.Sprintf(`Categorize this support ticket:
Subject: %s
Body: %s

Return JSON with:
- category: billing/technical/api/sender_id/campaign/account/other
- priority: low/medium/high/urgent
- sentiment: positive/neutral/negative
- suggested_response: brief template`, req.Subject, req.Body)

	resp, _ := s.llm.Complete(ctx, &llm.CompletionRequest{
		Messages: []llm.Message{{Role: "user", Content: prompt}},
	})

	s.jsonResponse(w, map[string]interface{}{
		"status":         "success",
		"categorization": json.RawMessage(resp.Content),
	}, http.StatusOK)
}

// ============== Analytics ==============

func (s *Service) handleSummarize(w http.ResponseWriter, r *http.Request) {
	ctx := r.Context()
	var req struct {
		AccountID string `json:"account_id"`
		Period    string `json:"period"` // daily, weekly, monthly
	}
	json.NewDecoder(r.Body).Decode(&req)

	// Gather stats
	var totalSent, delivered, failed int
	var spent float64
	s.db.QueryRow(ctx, `
		SELECT COUNT(*), 
			   SUM(CASE WHEN status='delivered' THEN 1 ELSE 0 END),
			   SUM(CASE WHEN status='failed' THEN 1 ELSE 0 END),
			   COALESCE(SUM(rate_per_sms), 0)
		FROM sms_history 
		WHERE account_id = $1 AND sent_date >= CURRENT_DATE - INTERVAL '7 days'
	`, req.AccountID).Scan(&totalSent, &delivered, &failed, &spent)

	prompt := fmt.Sprintf(`Create an executive summary for this SMS platform usage:
- Total sent: %d
- Delivered: %d (%.1f%%)
- Failed: %d
- Total spent: ₦%.2f
Period: %s

Provide insights on:
1. Performance overview
2. Key trends
3. Recommendations for next period

Keep it concise (3-4 paragraphs).`, totalSent, delivered,
		float64(delivered)/float64(totalSent)*100, failed, spent, req.Period)

	resp, _ := s.llm.Complete(ctx, &llm.CompletionRequest{
		Messages: []llm.Message{{Role: "user", Content: prompt}},
	})

	s.jsonResponse(w, map[string]interface{}{
		"status":  "success",
		"summary": resp.Content,
		"stats": map[string]interface{}{
			"total_sent": totalSent, "delivered": delivered,
			"failed": failed, "spent": spent,
		},
	}, http.StatusOK)
}

func (s *Service) handleAccountInsights(w http.ResponseWriter, r *http.Request) {
	accountID := chi.URLParam(r, "account_id")
	ctx := r.Context()

	// Quick stats
	var balance float64
	var campaigns, templates int
	s.db.QueryRow(ctx, "SELECT balance FROM accounts WHERE id = $1", accountID).Scan(&balance)
	s.db.QueryRow(ctx, "SELECT COUNT(*) FROM campaigns WHERE account_id = $1", accountID).Scan(&campaigns)
	s.db.QueryRow(ctx, "SELECT COUNT(*) FROM sms_templates WHERE account_id = $1", accountID).Scan(&templates)

	s.jsonResponse(w, map[string]interface{}{
		"account_id":   accountID,
		"balance":      balance,
		"campaigns":    campaigns,
		"templates":    templates,
		"generated_at": time.Now().UTC(),
	}, http.StatusOK)
}

// ============== Chat Interface ==============

func (s *Service) handleChat(w http.ResponseWriter, r *http.Request) {
	ctx := r.Context()
	var req struct {
		Messages []llm.Message `json:"messages"`
		Stream   bool          `json:"stream"`
	}
	json.NewDecoder(r.Body).Decode(&req)

	systemMsg := llm.Message{
		Role: "system",
		Content: `You are an AI assistant for the Brivas SMS platform. Help users with:
- Composing and improving SMS messages
- Understanding platform features
- Campaign planning and optimization
- Troubleshooting issues
Be helpful, concise, and professional.`,
	}
	messages := append([]llm.Message{systemMsg}, req.Messages...)

	resp, err := s.llm.Complete(ctx, &llm.CompletionRequest{Messages: messages})
	if err != nil {
		s.jsonError(w, "chat failed", http.StatusInternalServerError)
		return
	}

	s.jsonResponse(w, map[string]interface{}{
		"status":  "success",
		"message": resp.Content,
		"usage":   resp.Usage,
	}, http.StatusOK)
}

// Helpers

func (s *Service) jsonResponse(w http.ResponseWriter, data interface{}, status int) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	json.NewEncoder(w).Encode(data)
}

func (s *Service) jsonError(w http.ResponseWriter, msg string, status int) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	json.NewEncoder(w).Encode(map[string]string{"error": msg})
}
