# bitMaskRGB ↔ RGB Lightning Node Integration Context

## Service Architecture
- **bitMaskRGB**: Telegram bot service handling user interactions, Redis cache, HSM for virtual pubkeys
- **RGB Lightning Node**: Master Lightning Network node with RGB asset support
- **Database**: Shared PostgreSQL database (`rgb_dex_bot`) as settlement ledger
- **Redis**: Session cache and balance cache
- **Kafka**: Event streaming for inbound Lightning payments

## Three Payment Flows

### 💰 Flow 1: Internal Transfer (User A → User B)
**Goal**: Instant, zero-cost internal transfers
```
/send @userB 5000 sats → Database atomic transaction → Instant confirmation
```
- ✅ **Current Status**: IMPLEMENTED via `/virtual_transfer` endpoint
- Uses PostgreSQL as settlement ledger
- Virtual node IDs: `vn_{telegram_id}`
- UTXO-based balance management

### ⚡ Flow 2: External Payment (User A → External Node)
**Goal**: Route payment through Lightning using Master Node
```
/send @iris_user 10000 sats → Lightning Node API → Master node signs & routes → External payment
```
- ✅ **Current Status**: PARTIALLY IMPLEMENTED via `/virtual_sendpayment` endpoint
- Master node handles actual Lightning Network operations
- HSM provides virtual pubkeys (no signing for LN transactions)
- Refund on payment failure

### 📡 Flow 3: Inbound Payment (External Node → Internal User)
**Goal**: Receive external Lightning payments for internal users
```
External payment → Master Node → Kafka event → Bot processes → Credit user balance
```
- ❌ **Current Status**: NOT YET IMPLEMENTED
- Requires Kafka integration
- Invoice generation with user metadata
- Automatic balance crediting

## API Endpoints (RGB Lightning Node)
- `/virtual_transfer` - ✅ Internal transfers (Flow 1)
- `/virtual_sendpayment` - ✅ External payments (Flow 2) 
- `/virtual_rgbinvoice` - ✅ Invoice generation (Flow 3 prep)
- `/virtual_assetbalance` - ✅ Balance queries

## Database Schema
- `user_utxos` - Real Bitcoin UTXOs (telegram_id as user_id)
- `virtual_transactions` - Internal transfer audit trail
- `virtual_nodes` - Virtual pubkey mappings (HSM-derived)
- `ln_user_transactions` - All transaction history
- `ln_user_balances` - Cached balance data

## Key Integration Points
1. **User Authentication**: Redis session cache → PostgreSQL user profiles
2. **HSM Integration**: Virtual pubkey derivation for all users
3. **Balance Management**: PostgreSQL as source of truth, Redis as cache
4. **Transaction Recording**: Atomic database operations
5. **Lightning Integration**: Master node for external routing
6. **Event Processing**: Kafka for inbound payment notifications

## Implementation Status
- ✅ **Flow 1 (Internal)**: Complete with UTXO management
- 🔄 **Flow 2 (Outbound)**: Lightning router needs enhancement
- ❌ **Flow 3 (Inbound)**: Kafka integration required

## Next Steps for bitMaskRGB
1. Implement `/send` command handler
2. Add Redis session management
3. Create HSM integration for virtual pubkeys
4. Add balance caching logic
5. Implement Kafka consumer for inbound payments

## Service Communication Flows
```
# Flow 1: Internal Transfer
bitMaskRGB → POST /virtual_transfer → RGB Lightning Node → PostgreSQL → Response

# Flow 2: External Payment  
bitMaskRGB → POST /virtual_sendpayment → RGB Lightning Node → Lightning Network → Response

# Flow 3: Inbound Payment
External Node → Lightning Network → RGB Lightning Node → Kafka → bitMaskRGB → PostgreSQL
```