-- Add virtual_node_id column to ln_user_wallets table
ALTER TABLE ln_user_wallets ADD COLUMN IF NOT EXISTS virtual_node_id TEXT;

-- Create virtual_channels table
CREATE TABLE IF NOT EXISTS virtual_channels (
    id SERIAL PRIMARY KEY,
    channel_id TEXT NOT NULL,
    virtual_node_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(channel_id)
);

-- Create virtual_payments table
CREATE TABLE IF NOT EXISTS virtual_payments (
    id SERIAL PRIMARY KEY,
    payment_hash TEXT NOT NULL,
    virtual_node_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    inbound BOOLEAN NOT NULL,
    created_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(payment_hash)
);

-- Create indexes for better performance
CREATE INDEX IF NOT EXISTS idx_virtual_channels_virtual_node_id ON virtual_channels(virtual_node_id);
CREATE INDEX IF NOT EXISTS idx_virtual_channels_user_id ON virtual_channels(user_id);
CREATE INDEX IF NOT EXISTS idx_virtual_payments_virtual_node_id ON virtual_payments(virtual_node_id);
CREATE INDEX IF NOT EXISTS idx_virtual_payments_user_id ON virtual_payments(user_id);