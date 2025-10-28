#!/bin/bash

# Function to attempt unlock with retry
unlock_with_retry() {
    local max_attempts=5
    local attempt=1
    local delay=10
    
    while [ $attempt -le $max_attempts ]; do
        echo "Unlock attempt $attempt of $max_attempts..."
        
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
        
        if echo "$response" | grep -q "Connection pool timed out"; then
            echo "Connection pool timeout, retrying in ${delay} seconds..."
            sleep $delay
            attempt=$((attempt + 1))
            delay=$((delay + 5)) # Increase delay each attempt
        else
            echo "Unlock response: $response"
            if echo "$response" | grep -q '"error"'; then
                echo "Unlock failed with different error"
                exit 1
            else
                echo "Unlock successful!"
                exit 0
            fi
        fi
    done
    
    echo "All unlock attempts failed"
    exit 1
}

# Kill existing node
pkill -f rgb-lightning-node 2>/dev/null || true
sleep 2

# Start node
echo "Starting RGB Lightning Node..."
nohup target/debug/rgb-lightning-node /home/oem/.lighening-node-storage --daemon-listening-port 3001 --network testnet > node_retry.log 2>&1 &

# Wait for node to be ready
echo "Waiting for node to be ready..."
sleep 5

# Check if node is responding
until curl -s http://localhost:3001/networkinfo > /dev/null 2>&1; do
    echo "Waiting for node to respond..."
    sleep 2
done

echo "Node is ready, attempting unlock with retry mechanism..."
unlock_with_retry