#!/bin/bash

# Test synchronized virtual transfer using real telegram_ids from database
echo "🔄 Testing Synchronized Virtual Transfer"
echo "========================================"

# Use real telegram_ids from the database
FROM_USER=5984120724  # User with 2,800 sats
TO_USER=6512903955    # User with 159,787 sats
AMOUNT=500           # Transfer 500 sats

echo "📊 Before Transfer - Checking Balances:"
echo "User $FROM_USER balance:"
psql "postgresql://rgbit_bot_usr:!amadiohaDoings25@localhost:5432/rgb_dex_bot" -c "SELECT user_id, SUM(amount) as balance FROM user_utxos WHERE user_id = $FROM_USER AND spent = false GROUP BY user_id;"

echo "User $TO_USER balance:"
psql "postgresql://rgbit_bot_usr:!amadiohaDoings25@localhost:5432/rgb_dex_bot" -c "SELECT user_id, SUM(amount) as balance FROM user_utxos WHERE user_id = $TO_USER AND spent = false GROUP BY user_id;"

echo ""
echo "🚀 Executing Virtual Transfer: $FROM_USER → $TO_USER ($AMOUNT sats)"
echo "=================================================="

# Execute virtual transfer
curl -X POST -H "Content-Type: application/json" \
  -d "{\"from_user_id\": $FROM_USER, \"to_user_id\": $TO_USER, \"amount_sats\": $AMOUNT}" \
  http://localhost:3001/virtual_transfer

echo ""
echo ""
echo "📊 After Transfer - Checking Results:"
echo "====================================="

echo "User $FROM_USER balance:"
psql "postgresql://rgbit_bot_usr:!amadiohaDoings25@localhost:5432/rgb_dex_bot" -c "SELECT user_id, SUM(amount) as balance FROM user_utxos WHERE user_id = $FROM_USER AND spent = false GROUP BY user_id;"

echo "User $TO_USER balance:"
psql "postgresql://rgbit_bot_usr:!amadiohaDoings25@localhost:5432/rgb_dex_bot" -c "SELECT user_id, SUM(amount) as balance FROM user_utxos WHERE user_id = $TO_USER AND spent = false GROUP BY user_id;"

echo ""
echo "📝 Virtual Transactions:"
psql "postgresql://rgbit_bot_usr:!amadiohaDoings25@localhost:5432/rgb_dex_bot" -c "SELECT * FROM virtual_transactions ORDER BY created_at DESC LIMIT 3;"

echo ""
echo "🏦 Virtual Nodes:"
psql "postgresql://rgbit_bot_usr:!amadiohaDoings25@localhost:5432/rgb_dex_bot" -c "SELECT * FROM virtual_nodes ORDER BY created_at DESC LIMIT 5;"

echo ""
echo "💳 LN User Transactions:"
psql "postgresql://rgbit_bot_usr:!amadiohaDoings25@localhost:5432/rgb_dex_bot" -c "SELECT user_id, txid, amount, status, created_at FROM ln_user_transactions WHERE user_id IN ('$FROM_USER', '$TO_USER') ORDER BY created_at DESC LIMIT 5;"