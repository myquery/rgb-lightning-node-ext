#!/bin/bash

echo "🔧 Setting up Signet RGB Lightning Node..."

# Kill existing processes
pkill -f rgb-lightning-node 2>/dev/null || true
sleep 2

# Start regtest services (they work for signet too)
echo "🐳 Starting regtest services for signet..."
./regtest.sh start

# Wait for services
echo "⏳ Waiting for services to be ready..."
sleep 10

# Start node on signet with regtest services
echo "🚀 Starting RGB Lightning Node on signet..."
cargo run -- /home/oem/.lighening-node-storage-signet --daemon-listening-port 3001 --network signet > signet_node.log 2>&1 &

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
    echo "Mnemonic: $(echo $init_response | jq -r .mnemonic)"
elif echo "$init_response" | grep -q "already initialized"; then
    echo "✅ Wallet already initialized"
else
    echo "⚠️ Init response: $init_response"
fi

# Wait a bit more
sleep 5

# Unlock with regtest services
echo "🔓 Unlocking node..."
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

# Check final status
echo ""
echo "🔗 Final status:"
curl -s http://localhost:3001/nodeinfo

echo ""
echo "🎉 Signet setup complete!"
echo "💡 Test virtual API: curl -X POST -H 'Content-Type: application/json' -d '{\"asset_id\":\"test\"}' http://localhost:3001/virtual_rgbinvoice"