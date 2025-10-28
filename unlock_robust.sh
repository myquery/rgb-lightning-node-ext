#!/bin/bash

echo "🔓 Unlocking RGB Lightning Node (robust version)..."

# Function to attempt unlock with retries
attempt_unlock() {
    local max_attempts=10
    local attempt=1
    
    while [ $attempt -le $max_attempts ]; do
        echo "🔑 Unlock attempt $attempt of $max_attempts..."
        
        response=$(curl -s -X POST -H "Content-type: application/json" \
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
        
        echo "Response: $response"
        
        if echo "$response" | grep -q "changing state"; then
            echo "⏳ Node still changing state, waiting 10 seconds..."
            sleep 10
            attempt=$((attempt + 1))
        elif echo "$response" | grep -q '"error"'; then
            echo "❌ Unlock failed with error"
            return 1
        else
            echo "✅ Unlock successful!"
            return 0
        fi
    done
    
    echo "❌ All unlock attempts failed"
    return 1
}

# Wait for node to be ready
echo "⏳ Waiting for node to be ready..."
sleep 20

# Check node status
max_checks=10
check_count=0

while [ $check_count -lt $max_checks ]; do
    status=$(curl -s http://localhost:3001/nodeinfo 2>/dev/null)
    
    if echo "$status" | grep -q "Node is locked"; then
        echo "✅ Node is in locked state, ready for unlock"
        break
    fi
    
    echo "⏳ Waiting for node to be ready... ($((check_count + 1))/$max_checks)"
    sleep 5
    check_count=$((check_count + 1))
done

if [ $check_count -eq $max_checks ]; then
    echo "❌ Node failed to be ready"
    exit 1
fi

# Attempt unlock with retries
attempt_unlock

echo ""
echo "🔗 Final status check:"
curl -s http://localhost:3001/nodeinfo