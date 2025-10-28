-- RGB library tables in PostgreSQL
-- Mirrors the SQLite schema from rgb-lib

CREATE TABLE IF NOT EXISTS rgb_seaql_migrations (
    version VARCHAR NOT NULL PRIMARY KEY,
    applied_at BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS rgb_txo (
    idx SERIAL PRIMARY KEY,
    txid VARCHAR NOT NULL,
    vout BIGINT NOT NULL,
    btc_amount VARCHAR NOT NULL,
    spent BOOLEAN NOT NULL,
    exists BOOLEAN NOT NULL,
    pending_witness BOOLEAN NOT NULL,
    UNIQUE(txid, vout)
);

CREATE TABLE IF NOT EXISTS rgb_media (
    idx SERIAL PRIMARY KEY,
    digest VARCHAR NOT NULL UNIQUE,
    mime VARCHAR NOT NULL
);

CREATE TABLE IF NOT EXISTS rgb_asset (
    idx SERIAL PRIMARY KEY,
    media_idx INTEGER REFERENCES rgb_media(idx) ON DELETE RESTRICT ON UPDATE CASCADE,
    id VARCHAR NOT NULL UNIQUE,
    schema SMALLINT NOT NULL,
    added_at BIGINT NOT NULL,
    details VARCHAR,
    issued_supply VARCHAR NOT NULL,
    name VARCHAR NOT NULL,
    precision SMALLINT NOT NULL,
    ticker VARCHAR,
    timestamp BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS rgb_batch_transfer (
    idx SERIAL PRIMARY KEY,
    txid VARCHAR,
    status SMALLINT NOT NULL,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    expiration BIGINT,
    min_confirmations SMALLINT NOT NULL
);

CREATE TABLE IF NOT EXISTS rgb_asset_transfer (
    idx SERIAL PRIMARY KEY,
    user_driven BOOLEAN NOT NULL,
    batch_transfer_idx INTEGER NOT NULL REFERENCES rgb_batch_transfer(idx) ON DELETE CASCADE ON UPDATE CASCADE,
    asset_id VARCHAR REFERENCES rgb_asset(id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS rgb_coloring (
    idx SERIAL PRIMARY KEY,
    txo_idx INTEGER NOT NULL REFERENCES rgb_txo(idx) ON DELETE RESTRICT ON UPDATE RESTRICT,
    asset_transfer_idx INTEGER NOT NULL REFERENCES rgb_asset_transfer(idx) ON DELETE RESTRICT ON UPDATE RESTRICT,
    type SMALLINT NOT NULL,
    assignment JSONB NOT NULL
);

CREATE TABLE IF NOT EXISTS rgb_transfer (
    idx SERIAL PRIMARY KEY,
    asset_transfer_idx INTEGER NOT NULL REFERENCES rgb_asset_transfer(idx) ON DELETE CASCADE ON UPDATE CASCADE,
    requested_assignment JSONB,
    incoming BOOLEAN NOT NULL,
    recipient_type JSONB,
    recipient_id VARCHAR,
    ack BOOLEAN,
    invoice_string VARCHAR
);

CREATE TABLE IF NOT EXISTS rgb_transport_endpoint (
    idx SERIAL PRIMARY KEY,
    transport_type SMALLINT NOT NULL,
    endpoint VARCHAR NOT NULL,
    UNIQUE(transport_type, endpoint)
);

CREATE TABLE IF NOT EXISTS rgb_transfer_transport_endpoint (
    idx SERIAL PRIMARY KEY,
    transfer_idx INTEGER NOT NULL REFERENCES rgb_transfer(idx) ON DELETE CASCADE ON UPDATE CASCADE,
    transport_endpoint_idx INTEGER NOT NULL REFERENCES rgb_transport_endpoint(idx) ON DELETE RESTRICT ON UPDATE RESTRICT,
    used BOOLEAN NOT NULL DEFAULT FALSE,
    UNIQUE(transfer_idx, transport_endpoint_idx)
);

CREATE TABLE IF NOT EXISTS rgb_token (
    idx SERIAL PRIMARY KEY,
    asset_idx INTEGER NOT NULL REFERENCES rgb_asset(idx) ON DELETE CASCADE ON UPDATE CASCADE,
    index BIGINT NOT NULL,
    ticker VARCHAR,
    name VARCHAR,
    details VARCHAR,
    embedded_media BOOLEAN NOT NULL,
    reserves BOOLEAN NOT NULL
);

CREATE TABLE IF NOT EXISTS rgb_token_media (
    idx SERIAL PRIMARY KEY,
    token_idx INTEGER NOT NULL REFERENCES rgb_token(idx) ON DELETE CASCADE ON UPDATE CASCADE,
    media_idx INTEGER NOT NULL REFERENCES rgb_media(idx) ON DELETE RESTRICT ON UPDATE RESTRICT,
    attachment_id SMALLINT
);

CREATE TABLE IF NOT EXISTS rgb_wallet_transaction (
    idx SERIAL PRIMARY KEY,
    txid VARCHAR NOT NULL,
    type SMALLINT NOT NULL
);

CREATE TABLE IF NOT EXISTS rgb_pending_witness_script (
    idx SERIAL PRIMARY KEY,
    script VARCHAR NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS rgb_backup_info (
    idx SERIAL PRIMARY KEY,
    last_backup_timestamp VARCHAR NOT NULL,
    last_operation_timestamp VARCHAR NOT NULL
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_rgb_coloring_assettransferidx_txoidx ON rgb_coloring (asset_transfer_idx, txo_idx);