#!/bin/bash

echo "🔓 Patient unlock approach..."

# Wait much longer for complete initialization
echo "⏳ Waiting 2 minutes for complete initialization..."
sleep 120

# Check if we can unlock now
echo "🔑 Attempting unlock after long wait..."
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

if echo "$response" | grep -q '"error"'; then
    echo "❌ Still failed after long wait"
    echo "🔍 Let's check what's in the logs..."
    tail -20 node.log
else
    echo "✅ Success!"
fi

echo ""
echo "🔗 Final status:"
curl -s http://localhost:3001/nodeinfo