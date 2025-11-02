-- Create virtual_transactions table for recording transfers between virtual nodes
CREATE TABLE IF NOT EXISTS virtual_transactions (
    id SERIAL PRIMARY KEY,
    transaction_id VARCHAR(255) NOT NULL UNIQUE,
    from_virtual_node_id VARCHAR(255) NOT NULL,
    to_virtual_node_id VARCHAR(255) NOT NULL,
    from_user_id BIGINT NOT NULL,
    to_user_id BIGINT NOT NULL,
    amount_sats BIGINT NOT NULL,
    asset_id VARCHAR(255) NOT NULL DEFAULT 'BTC',
    status VARCHAR(50) NOT NULL DEFAULT 'completed',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes for better performance
CREATE INDEX IF NOT EXISTS idx_virtual_transactions_from_node ON virtual_transactions(from_virtual_node_id);
CREATE INDEX IF NOT EXISTS idx_virtual_transactions_to_node ON virtual_transactions(to_virtual_node_id);
CREATE INDEX IF NOT EXISTS idx_virtual_transactions_users ON virtual_transactions(from_user_id, to_user_id);
CREATE INDEX IF NOT EXISTS idx_virtual_transactions_tx_id ON virtual_transactions(transaction_id);