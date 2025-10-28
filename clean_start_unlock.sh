#!/bin/bash

echo "🧹 Clean start and unlock..."

# Kill everything on port 3001
echo "🛑 Cleaning up port 3001..."
lsof -ti:3001 | xargs kill -9 2>/dev/null || true
pkill -f rgb-lightning-node 2>/dev/null || true
sleep 5

# Start node fresh
echo "🚀 Starting node..."
target/debug/rgb-lightning-node /home/oem/.lighening-node-storage --daemon-listening-port 3001 --network testnet > node_clean.log 2>&1 &

# Wait for startup
echo "⏳ Waiting for startup..."
sleep 15

# Check if node is running
if ! curl -s http://localhost:3001/networkinfo > /dev/null 2>&1; then
    echo "❌ Node failed to start. Check logs:"
    tail -10 node_clean.log
    exit 1
fi

echo "✅ Node started successfully"

# Wait for full initialization
echo "⏳ Waiting for full initialization (60 seconds)..."
sleep 60

# Try unlock
echo "🔑 Unlocking..."
curl -X POST -H "Content-type: application/json" \
    -d '{
        "password": "testtest",
        "bitcoind_rpc_username": "user",
        "bitcoind_rpc_password": "password", 
        "bitcoind_rpc_host": "electrum.iriswallet.com",
        "bitcoind_rpc_port": 18332,
        "indexer_url": "ssl://electrum.iriswallet.com:50013",
        "proxy_endpoint": "rpcs://proxy.iriswallet.com/0.2/json-rpc",
        "announce_addresses": ["127.0.0.1:9735"]
    }' \
    http://localhost:3001/unlock

echo ""
echo "🔗 Status:"
curl -s http://localhost:3001/nodeinfo