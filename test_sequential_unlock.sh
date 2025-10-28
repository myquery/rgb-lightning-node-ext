#!/bin/bash

echo "Testing sequential initialization approach..."

# Kill existing node
pkill -f rgb-lightning-node 2>/dev/null || true
sleep 2

# Clean up any existing database files to start fresh
rm -rf /home/oem/.lighening-node-storage/rgb_sqlite 2>/dev/null || true

# Start node
echo "Starting RGB Lightning Node..."
nohup target/debug/rgb-lightning-node /home/oem/.lighening-node-storage --daemon-listening-port 3001 --network testnet > node_sequential.log 2>&1 &

# Wait for node to be ready
echo "Waiting for node to be ready..."
sleep 5

# Check if node is responding
until curl -s http://localhost:3001/networkinfo > /dev/null 2>&1; do
    echo "Waiting for node to respond..."
    sleep 2
done

echo "Node is ready, attempting unlock with sequential initialization..."

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

if echo "$response" | grep -q '"error"'; then
    echo "Unlock failed"
    echo "Checking logs..."
    tail -20 node_sequential.log
    exit 1
else
    echo "Unlock successful!"
    echo "Testing multi-user address endpoint..."
    
    # Test multi-user address generation
    address_response=$(curl -s -X POST -H "Content-type: application/json" \
        -d '{"user_id": "test_user_123"}' \
        http://localhost:3001/address)
    
    echo "Address response: $address_response"
    exit 0
fi