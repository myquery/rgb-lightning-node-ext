#!/bin/bash

echo "🔄 Starting RGB Lightning Node on Signet..."

# Clean up
lsof -ti:3001 | xargs kill -9 2>/dev/null || true
pkill -f rgb-lightning-node 2>/dev/null || true
sleep 3

# Start on signet
echo "🚀 Starting node on signet..."
target/debug/rgb-lightning-node /home/oem/.lighening-node-storage-signet --daemon-listening-port 3001 --network signet > node_signet.log 2>&1 &

# Wait for startup
echo "⏳ Waiting for startup..."
sleep 10

# Check if running
if ! curl -s http://localhost:3001/networkinfo > /dev/null 2>&1; then
    echo "❌ Failed to start. Logs:"
    tail -10 node_signet.log
    exit 1
fi

echo "✅ Node started on signet"
echo "⏳ Waiting 30 seconds for initialization..."
sleep 30

# Unlock with signet config
echo "🔑 Unlocking on signet..."
curl -X POST -H "Content-type: application/json" \
    -d '{
        "password": "testtest",
        "bitcoind_rpc_username": "user",
        "bitcoind_rpc_password": "password", 
        "bitcoind_rpc_host": "localhost",
        "bitcoind_rpc_port": 38332,
        "indexer_url": "tcp://127.0.0.1:50001",
        "proxy_endpoint": "rpc://127.0.0.1:3000/json-rpc"
    }' \
    http://localhost:3001/unlock

echo ""
echo "🔗 Status:"
curl -s http://localhost:3001/nodeinfo