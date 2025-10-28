#!/bin/bash

# Unlock script for deployed RGB Lightning Node

SERVER_IP=${1:-"YOUR_SERVER_IP"}
PASSWORD=${2:-"testtest"}

echo "🔓 Unlocking RGB Lightning Node at $SERVER_IP..."

# Wait for node to be ready
echo "Waiting for node to be ready..."
until curl -s http://$SERVER_IP:3001/networkinfo > /dev/null 2>&1; do
    sleep 2
done

# Unlock the node
curl -X POST http://$SERVER_IP:3001/unlock \
  -H "Content-Type: application/json" \
  -d "{
    \"password\": \"$PASSWORD\",
    \"bitcoind_rpc_username\": \"user\",
    \"bitcoind_rpc_password\": \"password\",
    \"bitcoind_rpc_host\": \"electrum.iriswallet.com\",
    \"bitcoind_rpc_port\": 18332,
    \"indexer_url\": \"ssl://electrum.iriswallet.com:50013\",
    \"proxy_endpoint\": \"rpcs://proxy.iriswallet.com/0.2/json-rpc\",
    \"announce_addresses\": [\"$SERVER_IP:9735\"]
  }"

echo ""
echo "✅ Node should now be unlocked!"
echo "🔍 Check status: curl http://$SERVER_IP:3001/nodeinfo"