-- Virtual channels mapping table
CREATE TABLE IF NOT EXISTS virtual_channels (
    channel_id TEXT PRIMARY KEY,
    virtual_node_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Virtual payments mapping table  
CREATE TABLE IF NOT EXISTS virtual_payments (
    payment_hash TEXT PRIMARY KEY,
    virtual_node_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    inbound BOOLEAN NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes for efficient lookups
CREATE INDEX idx_virtual_channels_virtual_node_id ON virtual_channels(virtual_node_id);
CREATE INDEX idx_virtual_channels_user_id ON virtual_channels(user_id);
CREATE INDEX idx_virtual_payments_virtual_node_id ON virtual_payments(virtual_node_id);
CREATE INDEX idx_virtual_payments_user_id ON virtual_payments(user_id);