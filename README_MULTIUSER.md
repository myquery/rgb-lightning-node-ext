# Multi-User RGB Lightning Node

This RGB Lightning Node now supports multi-user operations with PostgreSQL state management.

## Setup

1. **Install PostgreSQL** (if not already installed):
```bash
# Ubuntu/Debian
sudo apt install postgresql postgresql-contrib

# Create database and user
sudo -u postgres createdb rgb_dex_bot
sudo -u postgres createuser rgb_user
sudo -u postgres psql -c "ALTER USER rgb_user WITH PASSWORD 'your_password';"
sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE rgb_dex_bot TO rgb_user;"
```

2. **Configure Environment**:
```bash
cp .env.example .env
# Edit .env with your database credentials
DATABASE_URL=postgresql://rgb_user:your_password@localhost:5432/rgb_dex_bot
```

3. **Build and Run**:
```bash
cargo build --release
./target/release/rgb-lightning-node dataldk0/ --daemon-listening-port 3001 --ldk-peer-listening-port 9735 --network regtest
```

## Multi-User API Usage

All existing APIs now support user isolation by including a `user_id` parameter:

### Example API Calls

**Get User Address:**
```bash
curl -X POST http://localhost:3001/address \
  -H "Content-Type: application/json" \
  -d '{"user_id": "user123"}'
```

**Get User Balance:**
```bash
curl -X POST http://localhost:3001/btcbalance \
  -H "Content-Type: application/json" \
  -d '{"user_id": "user123", "skip_sync": false}'
```

**Get Asset Balance:**
```bash
curl -X POST http://localhost:3001/assetbalance \
  -H "Content-Type: application/json" \
  -d '{"user_id": "user123", "asset_id": "rgb:CJkb4YZw-jRiz2sk-~PARPio-wtVYI1c-XAEYCqO-wTfvRZ8"}'
```

### Alternative: Header-based User ID

You can also pass the user ID via HTTP header:
```bash
curl -X POST http://localhost:3001/address \
  -H "Content-Type: application/json" \
  -H "x-user-id: user123" \
  -d '{}'
```

## Database Schema

The node automatically creates the following tables:
- `ln_user_wallets` - User wallet information
- `ln_user_transactions` - User transaction history
- `ln_user_channels` - User Lightning channels
- `ln_user_balances` - User asset balances
- `ln_user_addresses` - User Bitcoin addresses

## Backward Compatibility

If no `DATABASE_URL` is provided, the node runs in single-user mode (original behavior).

## Integration with bitMaskRGB

This multi-user support is designed to work seamlessly with the bitMaskRGB project. The bitMaskRGB service can now:

1. Use the same PostgreSQL database (`rgb_dex_bot`)
2. Pass user IDs in API calls to isolate user operations
3. Track user-specific balances and transactions
4. Scale to millions of users with a single RGB Lightning Node

## Security Notes

- User isolation is enforced at the application layer
- Each user's data is stored separately in the database
- The underlying Lightning node still operates as a single entity
- Ensure proper authentication in your application layer before calling these APIs