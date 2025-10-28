#!/bin/bash

echo "Testing RGB Lightning Node without multi-user interference..."

# Kill existing node
pkill -f rgb-lightning-node 2>/dev/null || true
sleep 2

# Temporarily disable multi-user by renaming .env file
if [ -f ".env" ]; then
    mv .env .env.disabled
    echo "Temporarily disabled multi-user mode"
fi

# Clean up any existing database files
rm -rf /home/oem/.lighening-node-storage/rgb_sqlite 2>/dev/null || true

# Start node without multi-user features
echo "Starting RGB Lightning Node in pure single-user mode..."
nohup target/debug/rgb-lightning-node /home/oem/.lighening-node-storage --daemon-listening-port 3001 --network testnet > node_single_user.log 2>&1 &

# Wait for node to be ready
echo "Waiting for node to be ready..."
sleep 5

# Check if node is responding
until curl -s http://localhost:3001/networkinfo > /dev/null 2>&1; do
    echo "Waiting for node to respond..."
    sleep 2
done

echo "Node is ready, attempting unlock in pure single-user mode..."

# Attempt unlock
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

echo "Unlock response: $response"

# Restore .env file
if [ -f ".env.disabled" ]; then
    mv .env.disabled .env
    echo "Multi-user mode re-enabled"
fi

if echo "$response" | grep -q '"error"'; then
    echo "Unlock failed even in pure single-user mode - this is an RGB library issue"
    echo "Checking logs..."
    tail -20 node_single_user.log
    exit 1
else
    echo "Unlock successful in single-user mode!"
    echo "The issue is caused by multi-user database interference"
    exit 0
fi