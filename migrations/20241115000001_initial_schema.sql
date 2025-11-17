-- Initial database schema for Ping Buddy Bot
-- This migration creates the core tables for storing subscriptions and pending operations

-- Subscriptions table: tracks which users are subscribed to which topics in which chats
CREATE TABLE IF NOT EXISTS subscriptions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    chat_id INTEGER NOT NULL,
    topic TEXT NOT NULL,
    user_id TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(chat_id, topic, user_id)
);

-- Indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_subscriptions_chat_topic ON subscriptions(chat_id, topic);
CREATE INDEX IF NOT EXISTS idx_subscriptions_chat_user ON subscriptions(chat_id, user_id);
CREATE INDEX IF NOT EXISTS idx_subscriptions_topic ON subscriptions(topic);
CREATE INDEX IF NOT EXISTS idx_subscriptions_created_at ON subscriptions(created_at);

-- Pending topic creation table: tracks messages waiting for topic name replies
CREATE TABLE IF NOT EXISTS pending_topic_creation (
    chat_id INTEGER NOT NULL,
    message_id INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (chat_id, message_id)
);

-- Index for cleanup queries
CREATE INDEX IF NOT EXISTS idx_pending_created_at ON pending_topic_creation(created_at);

-- Metadata table: stores bot configuration and state
CREATE TABLE IF NOT EXISTS metadata (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Insert initial metadata
INSERT OR IGNORE INTO metadata (key, value) VALUES ('schema_version', '1');
INSERT OR IGNORE INTO metadata (key, value) VALUES ('created_at', datetime('now'));

