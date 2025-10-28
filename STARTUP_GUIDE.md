# RGB Lightning Node Startup Guide

## Quick Start

```bash
# Start the node (handles everything automatically)
./quick-start.sh
```

## Manual Startup Process

### 1. Environment Setup
```bash
# Ensure .env file exists with:
DATABASE_URL=postgresql://rgbit_bot_usr:!amadiohaDoings25@localhost:5432/rgb_dex_bot
BOT_TOKEN=your_telegram_bot_token_here
JWT_SECRET=rgb_lightning_node_jwt_secret_2024
```

### 2. Start Node Daemon
```bash
# Option A: Use startup script
./startup.sh

# Option B: Manual start
rgb-lightning-node dataldk0/ \
    --daemon-listening-port 3001 \
    --ldk-peer-listening-port 9735 \
    --network testnet
```

### 3. Unlock Node
```bash
# Option A: Use unlock script
./unlock.sh

# Option B: Manual unlock
curl -X POST http://localhost:3001/unlock \
  -H "Content-Type: application/json" \
  -d '{
    "password": "testtest",
    "bitcoind_rpc_username": "user",
    "bitcoind_rpc_password": "password",
    "bitcoind_rpc_host": "electrum.iriswallet.com",
    "bitcoind_rpc_port": 18332,
    "indexer_url": "ssl://electrum.iriswallet.com:50013",
    "proxy_endpoint": "rpcs://proxy.iriswallet.com/0.2/json-rpc",
    "announce_addresses": ["YOUR_SERVER_IP:9735"]
  }'
```

## Service Status Check

```bash
# Check if node is running
curl http://localhost:3001/networkinfo

# Check if node is unlocked
curl http://localhost:3001/nodeinfo

# Check multi-user database
curl http://localhost:3001/nodeinfo | grep -q "multi-user"
```

## Multi-User Features

After unlock, the following are automatically enabled:
- ✅ PostgreSQL database connection
- ✅ User manager initialization
- ✅ Telegram bot authentication
- ✅ Multi-user RGB wallet isolation
- ✅ JWT authentication service

## Telegram Bot Integration

The node automatically supports:
- Telegram user authentication
- Bot service proxy endpoints
- User-specific RGB wallets
- Isolated balances and transactions

## Troubleshooting

### Node won't start
```bash
# Check if port is available
netstat -tulpn | grep :3001

# Check logs
tail -f dataldk0/logs/rln.log
```

### Database connection issues
```bash
# Test database connection
psql postgresql://rgbit_bot_usr:!amadiohaDoings25@localhost:5432/rgb_dex_bot -c "SELECT 1;"
```

### Unlock fails
```bash
# Check if mnemonic exists
ls -la dataldk0/mnemonic

# Initialize if needed
curl -X POST http://localhost:3001/init \
  -H "Content-Type: application/json" \
  -d '{"password": "testtest"}'
```

## Production Deployment

1. Update `.env` with production values
2. Set proper `announce_addresses` in unlock call
3. Use strong passwords and JWT secrets
4. Configure firewall for ports 3001 and 9735
5. Set up SSL/TLS termination
6. Monitor logs and database connections