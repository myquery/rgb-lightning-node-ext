-- Add virtual_node_id column to user_wallets table
ALTER TABLE ln_user_wallets ADD COLUMN virtual_node_id TEXT;

-- Create index for efficient lookups
CREATE INDEX idx_ln_user_wallets_virtual_node_id ON ln_user_wallets(virtual_node_id);