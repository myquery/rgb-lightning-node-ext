#!/bin/bash

# Manual RGB Lightning Node Unlock Script

echo "🔓 Unlocking RGB Lightning Node..."

# Check if node is running
if ! curl -s http://localhost:3001/networkinfo > /dev/null 2>&1; then
    echo "❌ Node is not running. Start it first with ./startup.sh"
    exit 1
fi

# Unlock the node
curl -X POST http://localhost:3001/unlock \
  -H "Content-Type: application/json" \
  -d '{
    "password": "testtest",
    "bitcoind_rpc_username": "user",
    "bitcoind_rpc_password": "password",
    "bitcoind_rpc_host": "electrum.iriswallet.com",
    "bitcoind_rpc_port": 18332,
    "indexer_url": "ssl://electrum.iriswallet.com:50013",
    "proxy_endpoint": "rpcs://proxy.iriswallet.com/0.2/json-rpc",
    "announce_addresses": []
  }'

echo ""
echo "✅ Unlock request sent!"
echo "🔗 Check node status: curl http://localhost:3001/nodeinfo"