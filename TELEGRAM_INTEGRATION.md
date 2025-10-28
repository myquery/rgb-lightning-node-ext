# Telegram Bot Integration for RGB Lightning Node

## Overview

The RGB Lightning Node now supports Telegram bot authentication and service integration for the bitMaskRGB bot service.

## Authentication Methods

### 1. Telegram Login Widget Authentication
```rust
// POST /telegram/login
{
  "auth_data": {
    "id": 123456789,
    "first_name": "John",
    "last_name": "Doe",
    "username": "johndoe",
    "auth_date": 1640995200,
    "hash": "telegram_hash_here"
  }
}
```

### 2. Bot Service Authentication
```http
POST /api/endpoint
Headers:
  x-bot-token: YOUR_BOT_TOKEN
  x-telegram-user-id: 123456789
```

## User ID Format

Telegram users are identified with the format: `tg_{telegram_id}`
- Example: `tg_123456789`

## Bot Service Endpoints

### Bot Service Proxy
```http
POST /bot/service
Headers:
  Authorization: Bearer JWT_TOKEN
  x-bot-token: YOUR_BOT_TOKEN

Body:
{
  "telegram_user_id": 123456789,
  "telegram_username": "johndoe",
  "command": "get_balance",
  "data": {
    "asset_id": "optional_asset_id"
  }
}
```

### Supported Commands

1. **get_balance** - Get user's asset balance
2. **generate_address** - Generate new address for user
3. **send_asset** - Send assets to another user
4. **list_assets** - List user's assets
5. **create_channel** - Create Lightning channel

## Environment Variables

```bash
BOT_TOKEN=your_telegram_bot_token_here
DATABASE_URL=postgresql://user:pass@localhost/db
JWT_SECRET=your_jwt_secret_here
```

## Integration Flow

1. **User Authentication**: Telegram users authenticate via login widget or bot token
2. **User Context**: Each Telegram user gets isolated wallet and data
3. **Bot Requests**: bitMaskRGB bot makes authenticated requests on behalf of users
4. **Response**: RGB Lightning Node returns user-specific data

## Security Features

- Telegram auth data verification using bot token
- JWT token generation for session management
- User isolation and permission checking
- Bot token validation for service calls

## Example Usage from bitMaskRGB Bot

```javascript
// Authenticate user
const authResponse = await fetch('http://rgb-node:3001/telegram/login', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ auth_data: telegramAuthData })
});

// Make service call
const serviceResponse = await fetch('http://rgb-node:3001/bot/service', {
  method: 'POST',
  headers: {
    'Authorization': `Bearer ${token}`,
    'x-bot-token': process.env.BOT_TOKEN,
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({
    telegram_user_id: userId,
    telegram_username: username,
    command: 'get_balance',
    data: { asset_id: 'btc' }
  })
});
```

## Benefits

1. **Seamless Integration**: Direct integration with Telegram bot service
2. **User Isolation**: Each Telegram user has isolated RGB wallet
3. **Secure Authentication**: Verified Telegram auth data
4. **Flexible Commands**: Extensible command system for bot operations
5. **Multi-User Support**: Full multi-user capabilities for Telegram users