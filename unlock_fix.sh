#!/bin/bash

echo "🔧 RGB Lightning Node Unlock Fix"

# Function to check service connectivity
check_service() {
    local service=$1
    local host=$2
    local port=$3
    echo "🔍 Checking $service connectivity..."
    if timeout 5 nc -zv $host $port 2>/dev/null; then
        echo "✅ $service is reachable"
        return 0
    else
        echo "❌ $service is not reachable"
        return 1
    fi
}

# Function to test bitcoind RPC
test_bitcoind_rpc() {
    local host=$1
    local port=$2
    local user=$3
    local pass=$4
    echo "🔍 Testing bitcoind RPC..."
    response=$(curl -s --connect-timeout 5 --user $user:$pass \
        --data-binary '{"jsonrpc":"1.0","id":"test","method":"getblockchaininfo","params":[]}' \
        -H 'content-type: text/plain;' http://$host:$port/ 2>/dev/null)
    
    if echo "$response" | grep -q "result"; then
        echo "✅ Bitcoind RPC is working"
        return 0
    else
        echo "❌ Bitcoind RPC failed: $response"
        return 1
    fi
}

# Kill existing processes
echo "🧹 Cleaning up existing processes..."
pkill -f rgb-lightning-node 2>/dev/null || true
sleep 2

# Start regtest services
echo "🐳 Starting regtest services..."
./regtest.sh start

# Wait for services to be ready
echo "⏳ Waiting for services to initialize..."
sleep 15

# Verify all services are working
echo "🔍 Verifying service connectivity..."
check_service "Bitcoind" "localhost" "18433" || exit 1
check_service "Electrum" "127.0.0.1" "50001" || exit 1
check_service "RGB Proxy" "127.0.0.1" "3000" || exit 1

# Test bitcoind RPC specifically
test_bitcoind_rpc "localhost" "18433" "user" "password" || exit 1

# Start node
echo "🚀 Starting RGB Lightning Node..."
cargo run -- /home/oem/.lighening-node-storage-regtest --daemon-listening-port 3001 --network regtest > unlock_fix.log 2>&1 &
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
elif echo "$init_response" | grep -q "already initialized"; then
    echo "✅ Wallet already initialized"
else
    echo "⚠️  Wallet init response: $init_response"
fi

# Wait before unlock attempt
sleep 5

# Attempt unlock with retry logic
echo "🔓 Attempting to unlock node..."
max_retries=3
retry_count=0

while [ $retry_count -lt $max_retries ]; do
    echo "🔓 Unlock attempt $((retry_count + 1))/$max_retries..."
    
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
    
    # Check if unlock was successful
    if echo "$unlock_response" | grep -q "success\|unlocked\|true"; then
        echo "✅ Node unlocked successfully!"
        break
    elif echo "$unlock_response" | grep -q "Failed to connect to bitcoind"; then
        echo "❌ Bitcoind connection failed, retrying in 10 seconds..."
        sleep 10
        retry_count=$((retry_count + 1))
    else
        echo "❌ Unlock failed with unexpected response"
        retry_count=$((retry_count + 1))
        sleep 5
    fi
done

if [ $retry_count -ge $max_retries ]; then
    echo "❌ Failed to unlock after $max_retries attempts"
    echo "📋 Checking node logs..."
    tail -20 unlock_fix.log
    exit 1
fi

# Wait for unlock to complete
sleep 5

# Verify node status
echo ""
echo "🔗 Final Status Check:"
nodeinfo_response=$(curl -s http://localhost:3001/nodeinfo)
echo "$nodeinfo_response"

if echo "$nodeinfo_response" | grep -q "Node is locked"; then
    echo "❌ Node is still locked"
    exit 1
else
    echo "✅ Node unlock process completed!"
fi