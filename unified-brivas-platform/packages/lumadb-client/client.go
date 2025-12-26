// Package lumadb provides a direct wire protocol client for LumaDB
// LumaDB is a 100% drop-in replacement - no adapters needed
package lumadb

import (
	"context"
	"database/sql"
	"fmt"
	"sync"
	"time"

	_ "github.com/lib/pq" // PostgreSQL driver for LumaDB PostgreSQL wire protocol
)

// Client represents a connection to LumaDB using PostgreSQL wire protocol
type Client struct {
	db     *sql.DB
	config *Config
	mu     sync.RWMutex
}

// Config holds LumaDB connection configuration
type Config struct {
	Host            string
	Port            int
	Database        string
	User            string
	Password        string
	SSLMode         string
	MaxOpenConns    int
	MaxIdleConns    int
	ConnMaxLifetime time.Duration
	ConnMaxIdleTime time.Duration
}

// DefaultConfig returns sensible defaults for LumaDB connection
func DefaultConfig() *Config {
	return &Config{
		Host:            "localhost",
		Port:            5432, // LumaDB PostgreSQL wire protocol port
		Database:        "brivas",
		User:            "brivas",
		Password:        "",
		SSLMode:         "disable",
		MaxOpenConns:    100,
		MaxIdleConns:    25,
		ConnMaxLifetime: 5 * time.Minute,
		ConnMaxIdleTime: 1 * time.Minute,
	}
}

// Connect establishes a connection to LumaDB
// Uses standard PostgreSQL driver - LumaDB provides wire protocol compatibility
func Connect(cfg *Config) (*Client, error) {
	if cfg == nil {
		cfg = DefaultConfig()
	}

	connStr := fmt.Sprintf(
		"host=%s port=%d user=%s password=%s dbname=%s sslmode=%s",
		cfg.Host, cfg.Port, cfg.User, cfg.Password, cfg.Database, cfg.SSLMode,
	)

	db, err := sql.Open("postgres", connStr)
	if err != nil {
		return nil, fmt.Errorf("failed to open LumaDB connection: %w", err)
	}

	// Configure connection pool
	db.SetMaxOpenConns(cfg.MaxOpenConns)
	db.SetMaxIdleConns(cfg.MaxIdleConns)
	db.SetConnMaxLifetime(cfg.ConnMaxLifetime)
	db.SetConnMaxIdleTime(cfg.ConnMaxIdleTime)

	// Verify connection
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	if err := db.PingContext(ctx); err != nil {
		return nil, fmt.Errorf("failed to ping LumaDB: %w", err)
	}

	return &Client{
		db:     db,
		config: cfg,
	}, nil
}

// DB returns the underlying *sql.DB for direct SQL operations
// This enables seamless migration - existing SQL code works unchanged
func (c *Client) DB() *sql.DB {
	return c.db
}

// Close closes the database connection
func (c *Client) Close() error {
	c.mu.Lock()
	defer c.mu.Unlock()
	return c.db.Close()
}

// Exec executes a query without returning any rows
func (c *Client) Exec(ctx context.Context, query string, args ...interface{}) (sql.Result, error) {
	return c.db.ExecContext(ctx, query, args...)
}

// Query executes a query that returns rows
func (c *Client) Query(ctx context.Context, query string, args ...interface{}) (*sql.Rows, error) {
	return c.db.QueryContext(ctx, query, args...)
}

// QueryRow executes a query that returns at most one row
func (c *Client) QueryRow(ctx context.Context, query string, args ...interface{}) *sql.Row {
	return c.db.QueryRowContext(ctx, query, args...)
}

// BeginTx starts a transaction
func (c *Client) BeginTx(ctx context.Context, opts *sql.TxOptions) (*sql.Tx, error) {
	return c.db.BeginTx(ctx, opts)
}

// WithTransaction executes a function within a transaction
func (c *Client) WithTransaction(ctx context.Context, fn func(*sql.Tx) error) error {
	tx, err := c.db.BeginTx(ctx, nil)
	if err != nil {
		return fmt.Errorf("failed to begin transaction: %w", err)
	}

	if err := fn(tx); err != nil {
		if rbErr := tx.Rollback(); rbErr != nil {
			return fmt.Errorf("transaction failed: %w, rollback failed: %v", err, rbErr)
		}
		return err
	}

	return tx.Commit()
}

// Health checks the database connection health
func (c *Client) Health(ctx context.Context) error {
	return c.db.PingContext(ctx)
}

// Stats returns database connection pool statistics
func (c *Client) Stats() sql.DBStats {
	return c.db.Stats()
}
