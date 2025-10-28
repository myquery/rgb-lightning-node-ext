#!/bin/bash

echo "🔓 Unlocking RGB Lightning Node (with proper timing)..."

# Wait longer for node to fully initialize
echo "⏳ Waiting for node to complete initialization..."
sleep 15

# Wait for node to be completely ready (not in changing state)
max_checks=20
check_count=0

echo "🔍 Checking if node is ready for unlock..."
while [ $check_count -lt $max_checks ]; do
    response=$(curl -s http://localhost:3001/nodeinfo 2>/dev/null)
    
    if echo "$response" | grep -q "Node is locked"; then
        echo "✅ Node is ready for unlock (locked state)"
        break
    elif echo "$response" | grep -q "changing state"; then
        echo "⏳ Node still initializing... ($((check_count + 1))/$max_checks)"
    else
        echo "⏳ Waiting for node to be ready... ($((check_count + 1))/$max_checks)"
    fi
    
    sleep 5
    check_count=$((check_count + 1))
done

if [ $check_count -eq $max_checks ]; then
    echo "❌ Node failed to be ready after waiting"
    echo "Last response: $response"
    exit 1
fi

# Now attempt unlock
echo "🔑 Attempting to unlock node..."
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

echo "$response"

echo ""
if echo "$response" | grep -q '"error"'; then
    echo "❌ Unlock failed"
else
    echo "✅ Unlock successful!"
fi
echo "🔗 Check node status: curl http://localhost:3001/nodeinfo"