#!/bin/bash

echo "🔧 Simple RGB Lightning Node Unlock (No Docker)"

# Kill existing processes
echo "🧹 Cleaning up existing processes..."
pkill -f rgb-lightning-node 2>/dev/null || true
sleep 2

# Start node directly with testnet
echo "🚀 Starting RGB Lightning Node on testnet..."
cargo run -- /home/oem/.lighening-node-storage-testnet --daemon-listening-port 3001 --network testnet > simple_unlock.log 2>&1 &
NODE_PID=$!

# Wait for node startup with timeout
echo "⏳ Waiting for node to start..."
timeout=60
counter=0
while [ $counter -lt $timeout ]; do
    if curl -s http://localhost:3001/networkinfo > /dev/null 2>&1; then
        break
    fi
    sleep 2
    counter=$((counter + 2))
    echo "⏳ Still waiting... ($counter/$timeout seconds)"
done

if [ $counter -ge $timeout ]; then
    echo "❌ Node failed to start within $timeout seconds"
    kill $NODE_PID 2>/dev/null
    exit 1
fi

echo "✅ Node is responding"

# Initialize wallet if needed
echo "🔑 Checking wallet initialization..."
init_response=$(curl -s -X POST -H "Content-type: application/json" -d '{"password": "testtest"}' http://localhost:3001/init)

if echo "$init_response" | grep -q "mnemonic"; then
    echo "✅ Wallet initialized successfully"
    echo "📝 Mnemonic: $(echo "$init_response" | jq -r '.mnemonic // "Not found"')"
elif echo "$init_response" | grep -q "already initialized"; then
    echo "✅ Wallet already initialized"
else
    echo "⚠️  Wallet init response: $init_response"
fi

# Wait before unlock attempt
sleep 5

# Attempt unlock with testnet services
echo "🔓 Attempting to unlock node with testnet services..."

unlock_response=$(curl -s -X POST -H "Content-type: application/json" \
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
    http://localhost:3001/unlock)

echo "Unlock response: $unlock_response"

# Wait for unlock to complete
sleep 10

# Verify node status
echo ""
echo "🔗 Final Status Check:"
nodeinfo_response=$(curl -s http://localhost:3001/nodeinfo)
echo "$nodeinfo_response"

if echo "$nodeinfo_response" | grep -q "Node is locked"; then
    echo "❌ Node is still locked"
    echo "📋 Checking logs for errors..."
    tail -20 simple_unlock.log
    exit 1
else
    echo "✅ Node unlock process completed!"
    echo "🎉 RGB Lightning Node is ready for use!"
fi