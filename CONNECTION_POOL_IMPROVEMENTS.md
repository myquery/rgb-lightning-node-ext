# RGB Lightning Node - Connection Pool Improvements

## Problem Analysis

The original RGB Lightning Node was experiencing connection pool timeout errors in multi-user environments due to restrictive database connection settings in the underlying RGB library.

### Original Configuration (Problematic)
```rust
opt.max_connections(1)
    .min_connections(0)
    .connect_timeout(Duration::from_secs(8))
    .idle_timeout(Duration::from_secs(8))
    .max_lifetime(Duration::from_secs(8));
```

### Issues with Original Configuration
1. **Single Connection Limit**: Only 1 connection allowed, causing bottlenecks
2. **No Minimum Connections**: Pool could be completely empty
3. **Short Timeouts**: 8-second timeouts too aggressive for network operations
4. **No Acquire Timeout**: No timeout for acquiring connections from pool

## Solution: Extended RGB Library

We've created an extended version of the RGB library (`rgb-lib-extended`) with improved connection pool settings.

### New Configuration (Improved)
```rust
opt.max_connections(5)
    .min_connections(1)
    .connect_timeout(Duration::from_secs(30))
    .idle_timeout(Duration::from_secs(300))
    .max_lifetime(Duration::from_secs(3600))
    .acquire_timeout(Duration::from_secs(30));
```

### Benefits of New Configuration
1. **Multiple Connections**: Up to 5 concurrent connections
2. **Always Available**: Minimum 1 connection always maintained
3. **Generous Timeouts**: 30-second connection timeout for network delays
4. **Long-lived Connections**: 5-minute idle timeout, 1-hour max lifetime
5. **Acquire Protection**: 30-second timeout for acquiring connections

## Implementation

1. **Forked RGB Library**: Created `rgb-lib-extended` with improved settings
2. **Updated Dependencies**: Modified `Cargo.toml` to use local extended library
3. **Maintained Compatibility**: No API changes, only internal improvements

## Testing

Use the provided test script to verify the improvements:
```bash
./test_connection_pool.sh
```

## Deployment

The improved RGB Lightning Node can be deployed using the same methods as before, but with significantly better stability in multi-user environments.

### For Multi-User Scenarios
- Each user gets their own wallet directory
- Connection pool is shared efficiently
- Network delays are handled gracefully
- External services (bitcoind, indexer, proxy) work reliably

## Monitoring

Monitor the following metrics to verify improvements:
- Connection pool utilization
- Connection timeout errors (should be eliminated)
- Response times for wallet operations
- Concurrent user capacity