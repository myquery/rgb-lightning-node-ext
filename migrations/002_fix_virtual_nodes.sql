-- Add virtual_node_id column to existing virtual_channels table
ALTER TABLE virtual_channels ADD COLUMN IF NOT EXISTS virtual_node_id TEXT;

-- Create indexes for better performance
CREATE INDEX IF NOT EXISTS idx_virtual_channels_virtual_node_id ON virtual_channels(virtual_node_id);
CREATE INDEX IF NOT EXISTS idx_virtual_payments_virtual_node_id ON virtual_payments(virtual_node_id);
CREATE INDEX IF NOT EXISTS idx_virtual_payments_user_id ON virtual_payments(user_id);