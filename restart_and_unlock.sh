#!/bin/bash

echo "🔄 Restarting and unlocking RGB Lightning Node..."

# Kill any existing node processes
echo "🛑 Stopping existing node..."
pkill -f rgb-lightning-node 2>/dev/null || true
sleep 3

# Start fresh node
echo "🚀 Starting fresh node..."
nohup target/debug/rgb-lightning-node /home/oem/.lighening-node-storage --daemon-listening-port 3001 --network testnet > node.log 2>&1 &

# Wait for node to start
echo "⏳ Waiting for node to start..."
sleep 10

# Wait until node responds
echo "🔍 Waiting for node to respond..."
until curl -s http://localhost:3001/networkinfo > /dev/null 2>&1; do
    echo "⏳ Still waiting..."
    sleep 2
done

echo "✅ Node is responding"

# Wait a bit more for full initialization
echo "⏳ Waiting for full initialization..."
sleep 30

# Try unlock immediately
echo "🔑 Attempting unlock..."
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
echo "🔗 Node status:"
curl -s http://localhost:3001/nodeinfo