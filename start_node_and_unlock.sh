#!/bin/bash

# Kill any existing node
pkill -f rgb-lightning-node 2>/dev/null || true

# Start RGB Lightning Node in background
# Note: Database initialization happens AFTER unlock to avoid conflicts with RGB library
PATH="./bin:$PATH" CC=gcc RUSTFLAGS="-C linker=gcc" cargo run -- /home/oem/.lighening-node-storage --daemon-listening-port 3001 --network testnet &
NODE_PID=$!

# Wait until the API is ready
echo "Waiting for node to be ready on port 3001..."
until curl -s http://localhost:3001/networkinfo > /dev/null 2>&1; do
    sleep 1
done

echo "Node is up, sending unlock request..."

# Unlock the node
echo "Unlocking node..."
UNLOCK_RESPONSE=$(curl -s -X POST http://localhost:3001/unlock \
  -H "Content-Type: application/json" \
  -d '{
    "password": "testtest",
    "bitcoind_rpc_username": "user",
    "bitcoind_rpc_password": "password",
    "bitcoind_rpc_host": "electrum.iriswallet.com",
    "bitcoind_rpc_port": 18332,
    "indexer_url": "ssl://electrum.iriswallet.com:50013",
    "proxy_endpoint": "rpcs://proxy.iriswallet.com/0.2/json-rpc",
    "announce_addresses": ["127.0.0.1:9735"]
  }')

echo "Unlock response: $UNLOCK_RESPONSE"

# Wait for unlock to complete
echo "Waiting for node to unlock..."
until curl -s http://localhost:3001/nodeinfo > /dev/null 2>&1; do
    sleep 1
done

echo "Node unlocked successfully!"
echo "Multi-user database support initialized after unlock."
echo "Node running on port 3001 (PID: $NODE_PID)"
echo "You can now use the API endpoints like /address, /nodeinfo, etc."
echo "For multi-user mode, include 'user_id' in request body or 'x-user-id' header."
