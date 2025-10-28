# Database Separation for Multi-User Support

## Overview

This document explains how the RGB Lightning Node handles database separation to avoid conflicts between the RGB library's internal SQLite database and the PostgreSQL database used for multi-user support.

## Problem

The RGB library uses SQLite internally for its state management, while our multi-user implementation uses PostgreSQL for user isolation. When both databases try to initialize simultaneously during the unlock process, it can cause connection conflicts.

## Solution

### 1. Directory Separation

The RGB library's SQLite database is now stored in a separate subdirectory:

```
storage_dir/
├── rgb_sqlite/          # RGB library's SQLite database
│   ├── database.db
│   └── ...
├── postgresql/          # PostgreSQL connection (if configured)
└── ...                  # Other node data
```

### 2. Sequential Initialization

The initialization process follows this sequence:

1. **Node Unlock**: RGB library initializes its SQLite database in the `rgb_sqlite/` subdirectory
2. **Database Initialization**: PostgreSQL database connection is established after successful unlock
3. **User Manager Setup**: User management system is initialized with database connection

### 3. Environment Variables

For multi-user support, set the following environment variable:

```bash
export DATABASE_URL="postgresql://username:password@localhost/rgb_lightning_node"
```

If `DATABASE_URL` is not set, the node operates in single-user mode.

## Benefits

- **No Conflicts**: RGB library's SQLite and PostgreSQL operate in separate directories
- **Backward Compatibility**: Single-user mode works exactly as before
- **Clean Separation**: Each database system has its own dedicated space
- **Easy Maintenance**: Clear separation makes debugging and maintenance easier

## File Changes

- `src/ldk.rs`: Modified RGB wallet initialization to use `rgb_sqlite/` subdirectory
- `src/utils.rs`: Added database initialization after unlock
- `src/routes.rs`: Modified API endpoints to support user_id parameter

## Testing

To test the separation:

1. **Single-user mode**: Start node without `DATABASE_URL` - should work as before
2. **Multi-user mode**: Set `DATABASE_URL` and start node - should initialize both databases without conflicts
3. **Directory check**: Verify `rgb_sqlite/` directory is created and contains RGB library files