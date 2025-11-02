-- Add virtual_node_id column to ln_user_wallets table
ALTER TABLE ln_user_wallets ADD COLUMN IF NOT EXISTS virtual_node_id TEXT;

-- Create index for faster lookups
CREATE INDEX IF NOT EXISTS idx_ln_user_wallets_virtual_node_id ON ln_user_wallets(virtual_node_id);