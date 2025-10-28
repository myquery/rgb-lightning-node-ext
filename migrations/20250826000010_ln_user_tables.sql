-- User wallet state
CREATE TABLE IF NOT EXISTS ln_user_wallets (
    user_id TEXT PRIMARY KEY,
    mnemonic_encrypted TEXT NOT NULL,
    derivation_path TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- User transactions
CREATE TABLE IF NOT EXISTS ln_user_transactions (
    id UUID PRIMARY KEY,
    user_id TEXT NOT NULL,
    txid TEXT NOT NULL,
    amount BIGINT NOT NULL,
    asset_id TEXT,
    status TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- User channels
CREATE TABLE IF NOT EXISTS ln_user_channels (
    id UUID PRIMARY KEY,
    user_id TEXT NOT NULL,
    channel_id TEXT NOT NULL,
    peer_pubkey TEXT NOT NULL,
    capacity_sats BIGINT NOT NULL,
    status TEXT NOT NULL
);

-- User balances
CREATE TABLE IF NOT EXISTS ln_user_balances (
    user_id TEXT NOT NULL,
    asset_id TEXT,
    balance BIGINT NOT NULL DEFAULT 0,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Add unique constraint for user balances
CREATE UNIQUE INDEX IF NOT EXISTS idx_user_balances_unique ON ln_user_balances(user_id, COALESCE(asset_id, ''));

-- User addresses
CREATE TABLE IF NOT EXISTS ln_user_addresses (
    user_id TEXT NOT NULL,
    address TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    PRIMARY KEY (user_id, address)
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_user_transactions_user_id ON ln_user_transactions(user_id);
CREATE INDEX IF NOT EXISTS idx_user_channels_user_id ON ln_user_channels(user_id);
CREATE INDEX IF NOT EXISTS idx_user_balances_user_id ON ln_user_balances(user_id);
CREATE INDEX IF NOT EXISTS idx_user_addresses_user_id ON ln_user_addresses(user_id);