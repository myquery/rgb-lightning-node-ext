#!/bin/bash

# Test balance for specific testnet address
ADDRESS="tb1p2l333xrmymmx6jhur5ku0rw2sltdp774mthltguwt8hwsj8utxzsgx3st9"

echo "Testing balance query for address: $ADDRESS"
echo ""

# Use curl to test with a public testnet API as a demonstration
echo "Using public testnet API to check balance..."

# Test with blockstream testnet API
BALANCE_RESPONSE=$(curl -s "https://blockstream.info/testnet/api/address/$ADDRESS")

if [ $? -eq 0 ] && [ -n "$BALANCE_RESPONSE" ]; then
    echo "API Response:"
    echo "$BALANCE_RESPONSE" | jq '.'
    
    # Extract balance information
    FUNDED_TXOS=$(echo "$BALANCE_RESPONSE" | jq -r '.chain_stats.funded_txo_count // 0')
    SPENT_TXOS=$(echo "$BALANCE_RESPONSE" | jq -r '.chain_stats.spent_txo_count // 0')
    FUNDED_SUM=$(echo "$BALANCE_RESPONSE" | jq -r '.chain_stats.funded_txo_sum // 0')
    SPENT_SUM=$(echo "$BALANCE_RESPONSE" | jq -r '.chain_stats.spent_txo_sum // 0')
    
    BALANCE=$((FUNDED_SUM - SPENT_SUM))
    
    echo ""
    echo "Balance Summary:"
    echo "- Funded TXOs: $FUNDED_TXOS"
    echo "- Spent TXOs: $SPENT_TXOS"
    echo "- Total Received: $FUNDED_SUM satoshis"
    echo "- Total Spent: $SPENT_SUM satoshis"
    echo "- Current Balance: $BALANCE satoshis"
    
    if [ $BALANCE -gt 0 ]; then
        BTC_BALANCE=$(echo "scale=8; $BALANCE / 100000000" | bc -l)
        echo "- Current Balance: $BTC_BALANCE BTC"
    fi
else
    echo "Failed to query balance from public API"
    echo ""
    echo "The blockchain balance service would:"
    echo "1. Connect to your bitcoind node"
    echo "2. Use 'scantxoutset' RPC with addr($ADDRESS)"
    echo "3. Sum all unspent outputs for this address"
    echo "4. Return the total balance in satoshis"
fi

echo ""
echo "Note: This demonstrates how the blockchain balance service works."
echo "In production, it would use your local bitcoind node for more reliable queries."