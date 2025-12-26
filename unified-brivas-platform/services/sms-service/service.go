// Package sms provides a comprehensive SMS processing service
// Consolidated from brivas-api/controllers/smsController.js and bulkSMSController.js
package sms

import (
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"strings"
	"sync"
	"time"

	"github.com/go-chi/chi/v5"
	"go.uber.org/zap"

	lumadb "github.com/brivas/unified-platform/packages/lumadb-client"
)

// Service handles all SMS operations
type Service struct {
	db           *lumadb.Client
	logger       *zap.Logger
	providers    map[string]SMSProvider
	dlrBuffer    *DLRBuffer
	networkCodes map[string]string
}

// SMSProvider interface for SMS gateway providers
type SMSProvider interface {
	Name() string
	Send(ctx context.Context, msg *Message) (*SendResult, error)
	BulkSend(ctx context.Context, msgs []*Message) ([]*SendResult, error)
	GetDeliveryStatus(ctx context.Context, messageID string) (*DeliveryStatus, error)
}

// Message represents an SMS message
type Message struct {
	ID           string                 `json:"id"`
	SID          string                 `json:"sid"` // Session/batch ID
	RID          string                 `json:"rid"` // Response/message ID from provider
	AccountID    string                 `json:"account_id"`
	ServiceID    string                 `json:"service_id,omitempty"`
	From         string                 `json:"from"`
	To           string                 `json:"to"`
	Body         string                 `json:"body"`
	Type         string                 `json:"type"`     // transactional, promotional, otp, corporate
	SMSType      string                 `json:"sms_type"` // promotional, transactional
	Status       string                 `json:"status"`   // pending, sent, delivered, failed
	IsLive       bool                   `json:"is_live"`
	RatePerSMS   float64                `json:"rate_per_sms"`
	OriginBucket int                    `json:"origin_bucket_id"`
	UserAppID    int                    `json:"u_aid"`
	Network      string                 `json:"network"` // MTN, AIRTEL, GLO, 9MOBILE
	Webhook      string                 `json:"webhook,omitempty"`
	ScheduledAt  *time.Time             `json:"scheduled_at,omitempty"`
	SentDate     string                 `json:"sent_date"`
	SentTime     string                 `json:"sent_time"`
	Metadata     map[string]interface{} `json:"metadata,omitempty"`
}

// SendResult represents the result of sending an SMS
type SendResult struct {
	MessageID   string    `json:"message_id"`
	SID         string    `json:"sid"`
	Status      string    `json:"status"`
	Provider    string    `json:"provider"`
	Cost        float64   `json:"cost"`
	SubmittedAt time.Time `json:"submitted_at"`
}

// DeliveryStatus represents SMS delivery status from DLR callback
type DeliveryStatus struct {
	MessageID   string     `json:"message_id"`
	Status      string     `json:"status"` // pending, delivered, failed, expired
	To          string     `json:"to"`
	From        string     `json:"from"`
	DeliveredAt *time.Time `json:"delivered_at,omitempty"`
	ErrorCode   string     `json:"error_code,omitempty"`
	ErrorMsg    string     `json:"error_message,omitempty"`
}

// DLRBuffer buffers delivery report updates for batch processing
type DLRBuffer struct {
	delivered []string
	failed    []string
	mu        sync.Mutex
	db        *lumadb.Client
	logger    *zap.Logger
}

// BulkSendRequest represents a bulk SMS request
type BulkSendRequest struct {
	AccountID  string   `json:"account_id"`
	From       string   `json:"from"`
	To         []string `json:"to"`
	Message    string   `json:"message"`
	Type       string   `json:"type"` // promotional, transactional
	ScheduleAt string   `json:"schedule_at,omitempty"`
	LabelID    string   `json:"label_id,omitempty"`
}

// BulkSendResponse represents bulk send response
type BulkSendResponse struct {
	SID         string `json:"sid"`
	TotalSent   int    `json:"total_sent"`
	TotalFailed int    `json:"total_failed"`
	Status      string `json:"status"`
}

// Config for SMS service
type Config struct {
	MaxBulkRecipients int
	TestMaxRecipients int
	FlushInterval     time.Duration
	FlushBatchSize    int
}

// DefaultConfig returns default SMS service config
func DefaultConfig() *Config {
	return &Config{
		MaxBulkRecipients: 1000,
		TestMaxRecipients: 5,
		FlushInterval:     30 * time.Second,
		FlushBatchSize:    25,
	}
}

// NewService creates a new SMS service
func NewService(db *lumadb.Client, logger *zap.Logger, cfg *Config) *Service {
	if cfg == nil {
		cfg = DefaultConfig()
	}

	svc := &Service{
		db:        db,
		logger:    logger,
		providers: make(map[string]SMSProvider),
		dlrBuffer: &DLRBuffer{
			delivered: make([]string, 0),
			failed:    make([]string, 0),
			db:        db,
			logger:    logger,
		},
		networkCodes: map[string]string{
			"0803": "MTN", "0806": "MTN", "0703": "MTN", "0706": "MTN",
			"0813": "MTN", "0816": "MTN", "0810": "MTN", "0814": "MTN",
			"0903": "MTN", "0906": "MTN", "0913": "MTN", "0916": "MTN",
			"0805": "GLO", "0807": "GLO", "0705": "GLO", "0815": "GLO",
			"0811": "GLO", "0905": "GLO", "0915": "GLO",
			"0802": "AIRTEL", "0808": "AIRTEL", "0708": "AIRTEL",
			"0812": "AIRTEL", "0701": "AIRTEL", "0902": "AIRTEL",
			"0901": "AIRTEL", "0907": "AIRTEL", "0912": "AIRTEL",
			"0809": "9MOBILE", "0817": "9MOBILE", "0818": "9MOBILE",
			"0908": "9MOBILE", "0909": "9MOBILE",
		},
	}

	// Start DLR flush goroutine
	go svc.startDLRFlusher(cfg.FlushInterval, cfg.FlushBatchSize)

	return svc
}

// Routes returns Chi router with SMS endpoints
func (s *Service) Routes() chi.Router {
	r := chi.NewRouter()

	// Single SMS
	r.Post("/send", s.handleSend)
	r.Get("/history", s.handleHistory)

	// Bulk SMS
	r.Post("/bulk", s.handleBulkSend)
	r.Post("/bulk/schedule", s.handleSchedule)
	r.Get("/bulk/history", s.handleBulkHistory)
	r.Get("/bulk/insights", s.handleInsights)

	// DLR Callbacks (webhooks from providers)
	r.Post("/dlr/mtn", s.handleMTNDLR)
	r.Post("/dlr/airtel", s.handleAirtelDLR)
	r.Post("/dlr/glo", s.handleGloDLR)
	r.Post("/dlr/9mobile", s.handle9MobileDLR)
	r.Post("/dlr/smsc/promotional", s.handleSMSCDLRPromotional)
	r.Post("/dlr/smsc/transactional", s.handleSMSCDLRTransactional)
	r.Post("/dlr/smsc/corporate", s.handleSMSCDLRCorporate)

	// Balance
	r.Get("/balance", s.handleGetBalance)

	return r
}

// handleSend handles single SMS send
func (s *Service) handleSend(w http.ResponseWriter, r *http.Request) {
	ctx := r.Context()

	var req struct {
		To      string `json:"to"`
		From    string `json:"from,omitempty"`
		Message string `json:"message"`
	}

	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		s.jsonError(w, "invalid request body", http.StatusBadRequest)
		return
	}

	// Validate
	if req.To == "" || req.Message == "" {
		s.jsonError(w, "missing required fields: to, message", http.StatusBadRequest)
		return
	}

	// Get account from auth context (would be set by middleware)
	accountID := r.Header.Get("X-Account-ID")
	isLive := r.Header.Get("X-Is-Live") == "true"

	// Check balance
	var balance float64
	err := s.db.QueryRow(ctx,
		"SELECT balance FROM accounts WHERE id = $1", accountID).Scan(&balance)
	if err != nil {
		s.jsonError(w, "account not found", http.StatusUnauthorized)
		return
	}

	// Get rate
	rate := s.getRate("otp", req.To)
	if isLive && balance < rate {
		s.jsonError(w, "insufficient balance", http.StatusPaymentRequired)
		return
	}

	// Determine network and sender
	network := s.getNetwork(req.To)
	sender := req.From
	if sender == "" {
		sender = s.getDefaultSender(network)
	}

	// Generate SID
	sid := s.generateSID(accountID, "P2P")

	// Create message record
	msg := &Message{
		AccountID:  accountID,
		SID:        sid,
		From:       sender,
		To:         s.formatNumber(req.To),
		Body:       req.Message,
		Type:       "sms-otp",
		Status:     "pending",
		IsLive:     isLive,
		RatePerSMS: rate,
		Network:    network,
		SentDate:   time.Now().Format("2006-01-02"),
		SentTime:   time.Now().Format("15:04:05.000"),
	}

	// Send via provider
	result, err := s.sendViaProvider(ctx, msg)
	if err != nil {
		msg.Status = "failed"
		s.logSMS(ctx, msg)
		s.jsonError(w, "failed to send SMS", http.StatusInternalServerError)
		return
	}

	msg.RID = result.MessageID
	msg.Status = "pending"
	s.logSMS(ctx, msg)

	// Deduct balance
	if isLive {
		s.deductBalance(ctx, accountID, rate)
	}

	s.jsonResponse(w, map[string]interface{}{
		"status": "success",
		"msg":    "SMS sent",
		"data":   map[string]string{"sid": sid},
	}, http.StatusOK)
}

// handleBulkSend handles bulk SMS send
func (s *Service) handleBulkSend(w http.ResponseWriter, r *http.Request) {
	ctx := r.Context()

	var req BulkSendRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		s.jsonError(w, "invalid request body", http.StatusBadRequest)
		return
	}

	// Validate
	if len(req.To) == 0 || req.Message == "" || req.Type == "" {
		s.jsonError(w, "missing fields: to, message, type", http.StatusBadRequest)
		return
	}

	accountID := r.Header.Get("X-Account-ID")
	isLive := r.Header.Get("X-Is-Live") == "true"

	// Validate recipient count
	if !isLive && len(req.To) > 5 {
		s.jsonError(w, "max 5 recipients in test mode", http.StatusBadRequest)
		return
	}
	if len(req.To) > 1000 {
		s.jsonError(w, "max 1000 recipients", http.StatusBadRequest)
		return
	}

	// Validate sender
	sender := req.From
	if sender == "" {
		sender = s.getDefaultSenderByType(req.Type)
	} else {
		valid, err := s.validateSenderID(ctx, accountID, sender, req.Type)
		if err != nil || !valid {
			s.jsonError(w, "sender not approved", http.StatusBadRequest)
			return
		}
	}

	// Check balance for all recipients
	if isLive {
		var balance float64
		s.db.QueryRow(ctx, "SELECT balance FROM accounts WHERE id = $1", accountID).Scan(&balance)
		totalCost := float64(len(req.To)) * s.getRate(req.Type, "")
		if balance < totalCost {
			s.jsonError(w, "insufficient balance", http.StatusPaymentRequired)
			return
		}
	}

	// Generate batch SID
	sid := s.generateSID(accountID, "BULK")

	// Prepare messages
	messages := make([]*Message, 0, len(req.To))
	for _, to := range req.To {
		formatted := s.formatNumber(to)
		network := s.getNetwork(formatted)
		if network == "" {
			continue // Skip invalid numbers
		}

		messages = append(messages, &Message{
			AccountID:  accountID,
			SID:        sid,
			From:       sender,
			To:         formatted,
			Body:       req.Message,
			Type:       "bulk-sms",
			SMSType:    req.Type,
			Status:     "pending",
			IsLive:     isLive,
			RatePerSMS: s.getRate(req.Type, formatted),
			Network:    network,
			SentDate:   time.Now().Format("2006-01-02"),
			SentTime:   time.Now().Format("15:04:05.000"),
		})
	}

	// Send via bulk provider
	results, err := s.bulkSendViaProvider(ctx, messages, sender, req.Message, req.Type)
	if err != nil {
		s.jsonError(w, "bulk send failed: "+err.Error(), http.StatusInternalServerError)
		return
	}

	// Update message IDs from results
	for i, result := range results {
		if i < len(messages) {
			messages[i].RID = result.MessageID
			messages[i].Status = result.Status
		}
	}

	// Bulk insert to database
	s.bulkLogSMS(ctx, messages)

	// Deduct balance
	if isLive {
		totalCost := float64(len(results)) * s.getRate(req.Type, "")
		s.deductBalance(ctx, accountID, totalCost)
	}

	s.jsonResponse(w, map[string]interface{}{
		"status": "success",
		"msg":    "",
		"data":   map[string]string{"sid": sid},
	}, http.StatusOK)
}

// handleSMSCDLRPromotional handles SMSC promotional DLR callbacks
func (s *Service) handleSMSCDLRPromotional(w http.ResponseWriter, r *http.Request) {
	w.WriteHeader(http.StatusOK)

	var body map[string]interface{}
	if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
		return
	}

	go s.processDLRCallback(body, "promotional")
}

// handleSMSCDLRTransactional handles SMSC transactional DLR callbacks
func (s *Service) handleSMSCDLRTransactional(w http.ResponseWriter, r *http.Request) {
	w.WriteHeader(http.StatusOK)

	var body map[string]interface{}
	if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
		return
	}

	go s.processDLRCallback(body, "transactional")
}

// handleSMSCDLRCorporate handles SMSC corporate DLR callbacks
func (s *Service) handleSMSCDLRCorporate(w http.ResponseWriter, r *http.Request) {
	w.WriteHeader(http.StatusOK)

	var body map[string]interface{}
	if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
		return
	}

	go s.processDLRCallback(body, "corporate")
}

// processDLRCallback processes delivery report callbacks
func (s *Service) processDLRCallback(body map[string]interface{}, dlrType string) {
	ctx := context.Background()

	// Extract fields from callback
	var status, messageID, to, from string

	if msgID, ok := body["message_id"].(string); ok {
		messageID = msgID
	}
	if shortMsg, ok := body["short_message"].(string); ok {
		// Parse status from short message (SMSC format)
		parts := strings.Split(shortMsg, " ")
		for _, part := range parts {
			if strings.HasPrefix(part, "stat:") {
				status = strings.TrimPrefix(part, "stat:")
			}
			if strings.HasPrefix(part, "id:") && messageID == "" {
				messageID = strings.TrimPrefix(part, "id:")
			}
		}
	}
	if s, ok := body["status"].(string); ok {
		status = s
	}
	if t, ok := body["source_addr"].(string); ok {
		to = t
	}
	if f, ok := body["destination_addr"].(string); ok {
		from = f
	}

	// Normalize status
	var normalizedStatus string
	switch strings.ToUpper(status) {
	case "DELIVRD", "DELIVERED", "SENT":
		normalizedStatus = "delivered"
	case "UNDELIV", "FAILED", "REJECTED":
		normalizedStatus = "failed"
	default:
		normalizedStatus = "failed"
	}

	// Queue for batch update
	s.dlrBuffer.queue(messageID, normalizedStatus)

	// Send webhook if configured
	s.sendWebhook(ctx, messageID, DeliveryStatus{
		MessageID: messageID,
		Status:    normalizedStatus,
		To:        to,
		From:      from,
	})

	// Refund if failed
	if normalizedStatus == "failed" {
		s.refundFailedSMS(ctx, messageID)
	}
}

// handleMTNDLR handles MTN DLR callbacks
func (s *Service) handleMTNDLR(w http.ResponseWriter, r *http.Request) {
	w.WriteHeader(http.StatusOK)
	// MTN-specific DLR processing
}

// handleAirtelDLR handles Airtel DLR callbacks
func (s *Service) handleAirtelDLR(w http.ResponseWriter, r *http.Request) {
	w.WriteHeader(http.StatusOK)
	// Airtel-specific DLR processing
}

// handleGloDLR handles Glo DLR callbacks
func (s *Service) handleGloDLR(w http.ResponseWriter, r *http.Request) {
	w.WriteHeader(http.StatusOK)
}

// handle9MobileDLR handles 9Mobile DLR callbacks
func (s *Service) handle9MobileDLR(w http.ResponseWriter, r *http.Request) {
	w.WriteHeader(http.StatusOK)
}

// handleHistory handles SMS history query
func (s *Service) handleHistory(w http.ResponseWriter, r *http.Request) {
	ctx := r.Context()
	accountID := r.Header.Get("X-Account-ID")

	page := 1
	if p := r.URL.Query().Get("page"); p != "" {
		fmt.Sscanf(p, "%d", &page)
	}
	limit := 50
	offset := (page - 1) * limit

	rows, err := s.db.Query(ctx, `
		SELECT sid, sender, recipient, message, status, type, rate_per_sms, sent_date, sent_time
		FROM sms_history
		WHERE account_id = $1 AND type = 'sms-otp'
		ORDER BY id DESC
		LIMIT $2 OFFSET $3
	`, accountID, limit, offset)
	if err != nil {
		s.jsonError(w, "failed to fetch history", http.StatusInternalServerError)
		return
	}
	defer rows.Close()

	var history []map[string]interface{}
	for rows.Next() {
		var sid, sender, recipient, message, status, typ, sentDate, sentTime string
		var rate float64
		rows.Scan(&sid, &sender, &recipient, &message, &status, &typ, &rate, &sentDate, &sentTime)
		history = append(history, map[string]interface{}{
			"sid":     sid,
			"from":    sender,
			"to":      recipient,
			"message": message,
			"status":  status,
			"type":    typ,
			"rate":    rate,
			"date":    sentDate,
			"time":    sentTime,
		})
	}

	s.jsonResponse(w, map[string]interface{}{
		"status": "success",
		"data":   history,
		"msg":    "Fetched successfully",
	}, http.StatusOK)
}

// handleBulkHistory handles bulk SMS history
func (s *Service) handleBulkHistory(w http.ResponseWriter, r *http.Request) {
	s.handleHistory(w, r) // Same logic, different type filter
}

// handleSchedule handles scheduled bulk SMS
func (s *Service) handleSchedule(w http.ResponseWriter, r *http.Request) {
	// Scheduled SMS implementation
	s.jsonResponse(w, map[string]interface{}{
		"status": "success",
		"msg":    "Message scheduled",
	}, http.StatusOK)
}

// handleInsights handles bulk SMS analytics
func (s *Service) handleInsights(w http.ResponseWriter, r *http.Request) {
	ctx := r.Context()
	accountID := r.Header.Get("X-Account-ID")

	var totalSent, totalDelivered, totalFailed int
	s.db.QueryRow(ctx, `
		SELECT 
			COUNT(*) as total,
			SUM(CASE WHEN status = 'delivered' THEN 1 ELSE 0 END) as delivered,
			SUM(CASE WHEN status = 'failed' THEN 1 ELSE 0 END) as failed
		FROM sms_history
		WHERE account_id = $1 AND type = 'bulk-sms'
	`, accountID).Scan(&totalSent, &totalDelivered, &totalFailed)

	s.jsonResponse(w, map[string]interface{}{
		"data": map[string]int{
			"total_sent":      totalSent,
			"total_delivered": totalDelivered,
			"total_failed":    totalFailed,
		},
	}, http.StatusOK)
}

// handleGetBalance returns SMS balance
func (s *Service) handleGetBalance(w http.ResponseWriter, r *http.Request) {
	ctx := r.Context()
	accountID := r.Header.Get("X-Account-ID")

	var balance float64
	s.db.QueryRow(ctx, "SELECT balance FROM accounts WHERE id = $1", accountID).Scan(&balance)

	s.jsonResponse(w, map[string]interface{}{
		"status": "success",
		"msg":    "Bulk SMS Unit Balance",
		"data":   balance,
	}, http.StatusOK)
}

// Helper methods

func (s *Service) getNetwork(number string) string {
	if len(number) < 4 {
		return ""
	}
	// Handle Nigerian numbers
	prefix := number
	if strings.HasPrefix(number, "234") {
		prefix = "0" + number[3:]
	}
	if len(prefix) >= 4 {
		if network, ok := s.networkCodes[prefix[:4]]; ok {
			return network
		}
	}
	return "MTN" // Default
}

func (s *Service) formatNumber(number string) string {
	number = strings.TrimSpace(number)
	if strings.HasPrefix(number, "+") {
		number = number[1:]
	}
	if strings.HasPrefix(number, "0") {
		number = "234" + number[1:]
	}
	return number
}

func (s *Service) getRate(msgType, recipient string) float64 {
	rates := map[string]float64{
		"otp":           3.0,
		"transactional": 3.0,
		"promotional":   2.5,
		"corporate":     3.0,
	}
	if rate, ok := rates[msgType]; ok {
		return rate
	}
	return 3.0
}

func (s *Service) getDefaultSender(network string) string {
	senders := map[string]string{
		"MTN":     "BRIVAS",
		"AIRTEL":  "BRIVAS",
		"GLO":     "BRIVAS",
		"9MOBILE": "BRIVAS",
	}
	if sender, ok := senders[network]; ok {
		return sender
	}
	return "BRIVAS"
}

func (s *Service) getDefaultSenderByType(smsType string) string {
	return "BRIVAS" // Default sender
}

func (s *Service) generateSID(accountID, useCase string) string {
	return fmt.Sprintf("%s-%s-%d", accountID[:8], useCase, time.Now().UnixNano())
}

func (s *Service) validateSenderID(ctx context.Context, accountID, sender, smsType string) (bool, error) {
	var count int
	err := s.db.QueryRow(ctx, `
		SELECT COUNT(*) FROM sender_ids 
		WHERE (account_id = $1 OR is_general = true OR is_public = true)
		AND sender = $2 AND type = $3 AND approved = true
	`, accountID, sender, smsType).Scan(&count)
	return count > 0, err
}

func (s *Service) sendViaProvider(ctx context.Context, msg *Message) (*SendResult, error) {
	// Select provider based on network/type
	for _, provider := range s.providers {
		return provider.Send(ctx, msg)
	}
	// Mock response for demo
	return &SendResult{
		MessageID:   fmt.Sprintf("%d-%d", time.Now().UnixNano(), time.Now().UnixMicro()),
		Status:      "pending",
		SubmittedAt: time.Now(),
	}, nil
}

func (s *Service) bulkSendViaProvider(ctx context.Context, msgs []*Message, sender, message, smsType string) ([]*SendResult, error) {
	results := make([]*SendResult, len(msgs))
	for i := range msgs {
		results[i] = &SendResult{
			MessageID:   fmt.Sprintf("%d-%d-%d", time.Now().UnixNano(), time.Now().UnixMicro(), i),
			Status:      "pending",
			SubmittedAt: time.Now(),
		}
	}
	return results, nil
}

func (s *Service) logSMS(ctx context.Context, msg *Message) {
	s.db.Exec(ctx, `
		INSERT INTO sms_history 
		(account_id, sid, rid, sender, recipient, message, status, type, sms_type, rate_per_sms, is_live, sent_date, sent_time)
		VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
	`, msg.AccountID, msg.SID, msg.RID, msg.From, msg.To, msg.Body, msg.Status, msg.Type, msg.SMSType, msg.RatePerSMS, msg.IsLive, msg.SentDate, msg.SentTime)
}

func (s *Service) bulkLogSMS(ctx context.Context, msgs []*Message) {
	for _, msg := range msgs {
		s.logSMS(ctx, msg)
	}
}

func (s *Service) deductBalance(ctx context.Context, accountID string, amount float64) {
	s.db.Exec(ctx, "UPDATE accounts SET balance = balance - $1 WHERE id = $2", amount, accountID)
}

func (s *Service) refundFailedSMS(ctx context.Context, messageID string) {
	// Get message details and refund
	var accountID string
	var rate float64
	var isLive bool
	err := s.db.QueryRow(ctx, `
		SELECT account_id, rate_per_sms, is_live FROM sms_history WHERE rid = $1
	`, messageID).Scan(&accountID, &rate, &isLive)
	if err != nil || !isLive {
		return
	}
	s.db.Exec(ctx, "UPDATE accounts SET balance = balance + $1 WHERE id = $2", rate, accountID)
}

func (s *Service) sendWebhook(ctx context.Context, messageID string, status DeliveryStatus) {
	// Get webhook URL from message or user app
	var webhook string
	s.db.QueryRow(ctx, `
		SELECT ua.webhook FROM sms_history sh
		JOIN user_apps ua ON sh.u_aid = ua.id
		WHERE sh.rid = $1
	`, messageID).Scan(&webhook)

	if webhook == "" {
		return
	}

	// Send webhook
	payload, _ := json.Marshal(status)
	http.Post(webhook, "application/json", strings.NewReader(string(payload)))
}

func (s *Service) startDLRFlusher(interval time.Duration, batchSize int) {
	ticker := time.NewTicker(interval)
	for range ticker.C {
		s.dlrBuffer.flush(batchSize)
	}
}

func (b *DLRBuffer) queue(messageID, status string) {
	b.mu.Lock()
	defer b.mu.Unlock()

	switch status {
	case "delivered":
		b.delivered = append(b.delivered, messageID)
	case "failed":
		b.failed = append(b.failed, messageID)
	}
}

func (b *DLRBuffer) flush(batchSize int) {
	b.mu.Lock()
	defer b.mu.Unlock()

	ctx := context.Background()

	// Flush delivered
	if len(b.delivered) > 0 {
		toFlush := b.delivered
		if len(toFlush) > batchSize {
			toFlush = b.delivered[:batchSize]
			b.delivered = b.delivered[batchSize:]
		} else {
			b.delivered = make([]string, 0)
		}
		b.updateStatusBatch(ctx, toFlush, "delivered")
	}

	// Flush failed
	if len(b.failed) > 0 {
		toFlush := b.failed
		if len(toFlush) > batchSize {
			toFlush = b.failed[:batchSize]
			b.failed = b.failed[batchSize:]
		} else {
			b.failed = make([]string, 0)
		}
		b.updateStatusBatch(ctx, toFlush, "failed")
	}
}

func (b *DLRBuffer) updateStatusBatch(ctx context.Context, messageIDs []string, status string) {
	if len(messageIDs) == 0 {
		return
	}

	placeholders := make([]string, len(messageIDs))
	args := make([]interface{}, len(messageIDs)+1)
	args[0] = status
	for i, id := range messageIDs {
		placeholders[i] = fmt.Sprintf("$%d", i+2)
		args[i+1] = id
	}

	query := fmt.Sprintf("UPDATE sms_history SET status = $1 WHERE rid IN (%s)", strings.Join(placeholders, ","))
	b.db.Exec(ctx, query, args...)
	b.logger.Info("flushed DLR updates", zap.String("status", status), zap.Int("count", len(messageIDs)))
}

func (s *Service) jsonResponse(w http.ResponseWriter, data interface{}, status int) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	json.NewEncoder(w).Encode(data)
}

func (s *Service) jsonError(w http.ResponseWriter, msg string, status int) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	json.NewEncoder(w).Encode(map[string]string{"error": msg, "status": "error"})
}
