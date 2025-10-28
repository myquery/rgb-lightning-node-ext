#!/bin/bash

echo "🔧 Setting up RGB Lightning Node with regtest..."

# Kill existing processes
pkill -f rgb-lightning-node 2>/dev/null || true
sleep 2

# Start regtest services
echo "🐳 Starting regtest services..."
./regtest.sh start

# Wait for services to be ready
echo "⏳ Waiting for services..."
sleep 10

# Start node on regtest
echo "🚀 Starting RGB Lightning Node on regtest..."
cargo run -- /home/oem/.lighening-node-storage-regtest --daemon-listening-port 3001 --network regtest > regtest_node.log 2>&1 &

# Wait for node startup
echo "⏳ Waiting for node startup..."
sleep 15

# Check if node is responding
until curl -s http://localhost:3001/networkinfo > /dev/null 2>&1; do
    echo "⏳ Waiting for node to respond..."
    sleep 3
done

echo "✅ Node is responding"

# Initialize wallet if needed
echo "🔑 Initializing wallet..."
init_response=$(curl -s -X POST -H "Content-type: application/json" -d '{"password": "testtest"}' http://localhost:3001/init)

if echo "$init_response" | grep -q "mnemonic"; then
    echo "✅ Wallet initialized"
elif echo "$init_response" | grep -q "already initialized"; then
    echo "✅ Wallet already initialized"
fi

sleep 5

# Unlock with local regtest services
echo "🔓 Unlocking with local regtest services..."
unlock_response=$(curl -s -X POST -H "Content-type: application/json" \
    -d '{
        "password": "testtest",
        "bitcoind_rpc_username": "user",
        "bitcoind_rpc_password": "password", 
        "bitcoind_rpc_host": "localhost",
        "bitcoind_rpc_port": 18433,
        "indexer_url": "127.0.0.1:50001",
        "proxy_endpoint": "rpc://127.0.0.1:3000/json-rpc",
        "announce_addresses": ["127.0.0.1:9735"]
    }' \
    http://localhost:3001/unlock)

echo "Unlock response: $unlock_response"

# Wait a moment for unlock to complete
sleep 5

echo ""
echo "🔗 Status:"
curl -s http://localhost:3001/nodeinfo