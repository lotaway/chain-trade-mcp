# Implementation Plan

Based on `SPEC.md` requirements, this document outlines the plan to provide real, executable, and auditable on-chain and market capabilities for Web3 quantitative agents.

---

## P0 · Required Tools (Real Execution)

### ✅ onchain_balance (`get_balance`)
**Status: Completed**

Implemented in:
- `src/interface/tools/balance.rs`
- `src/domain/service/balance_service.rs`
- `src/infrastructure/repository/ethereum_repository.rs`

Features:
- ✅ Query ETH and ERC20 token balances
- ✅ Direct RPC calls
- ✅ Returns real-time balance data

---

### ✅ token_price (`get_token_price`)
**Status: Completed**

Implemented in:
- `src/interface/tools/price.rs`
- `src/domain/service/price_service.rs`
- `src/infrastructure/repository/ethereum_repository.rs`

Features:
- ✅ Real market price query via Uniswap V3 Quoter
- ✅ Returns latest available price in USDC
- ✅ Multi-source support (currently Uniswap V3)

---

### ⚠️ swap_tokens
**Status: Need Upgrade (Simulation → Real Execution)**

Current Implementation:
- `src/interface/tools/swap.rs` - Currently simulates swaps only
- `src/domain/service/swap_service.rs`
- `src/infrastructure/repository/ethereum_repository.rs` - `simulate_swap()` exists

Required Changes:
1. Add `PRIVATE_KEY` configuration validation
2. Implement `execute_swap()` method (not just simulation)
3. Add required parameters:
   - ✅ `dex` / router (hardcoded Uniswap V3 for now)
   - ✅ `slippage` (already supported)
   - Add `max_input` or `min_output` (optional with defaults)
4. Return real `tx_hash` on execution
5. Ensure failure returns error (no silent failure)
6. Add gas limit and max fee validation (safety)

Target Signature:
```json
{
  "from_token": "0x...",
  "to_token": "0x...",
  "amount": "1.0",
  "slippage": 0.5,
  "max_spend": "1.1",        // Optional: max input
  "min_receive": "1800"       // Optional: min output
}
```

---

## P1 · Recommended Tools (Real Data)

### 📋 news_search
**Status: Pending**

Purpose: Real news source retrieval for market sentiment

Implementation Plan:
1. Create `NewsService` in `src/domain/service/`
2. Add news API client (RSS / public API)
3. Create `NewsTool` in `src/interface/tools/`
4. Return raw content without summarization

Input Schema:
```json
{
  "query": "ethereum",
  "limit": 10,
  "source": "rss"  // Optional: specify source
}
```

Output: Raw news articles (no scoring, no summarization)

---

### 📋 onchain_transfer_monitor
**Status: Pending**

Purpose: Real on-chain transfer record query for monitoring abnormal fund flows

Implementation Plan:
1. Create `TransferRepository` in `src/domain/repository/`
2. Add transfer query methods to `EthereumClient`
3. Create `TransferMonitorTool` in `src/interface/tools/`
4. Return raw transfer data without address label inference

Input Schema:
```json
{
  "address": "0x...",
  "from_block": 18000000,
  "to_block": 18000100
}
```

Output: Raw transfer events (from, to, value, hash, block)

---

### 📋 market_volume
**Status: Pending**

Purpose: Real trading volume / price changes for liquidity & volatility filtering

Implementation Plan:
1. Create `MarketDataService` in `src/domain/service/`
2. Integrate with DEX APIs (Uniswap, etc.)
3. Create `MarketVolumeTool` in `src/interface/tools/`

Input Schema:
```json
{
  "token_address": "0x...",
  "time_range": "24h"  // 1h, 24h, 7d
}
```

Output: Volume, price change percentage, liquidity info

---

## P2 · Optional Tools

### 📋 equity_price
**Status: Pending**

Purpose: Real stock / index prices (query only, does not affect on-chain execution)

Implementation Plan:
1. Create `EquityPriceService` in `src/domain/service/`
2. Add stock API integration (Yahoo Finance, Alpha Vantage, etc.)
3. Create `EquityPriceTool` in `src/interface/tools/`

Input Schema:
```json
{
  "symbol": "AAPL",
  "exchange": "NASDAQ"  // Optional
}
```

Output: Current stock price

---

## System Invariants - Verification Checklist

- [x] Query tools are read-only
- [ ] Execution tools require explicit authorization (PRIVATE_KEY config validation)
- [x] Failures must return errors
- [x] No silent failure allowed
- [x] Same input produces deterministic behavior
- [ ] Any execution tool produces at most one on-chain transaction
- [ ] Tools must not implicitly modify transaction parameters
- [ ] Agent cannot modify gas limit ceiling
- [ ] Agent cannot bypass slippage
- [ ] Agent cannot initiate infinite approval

---

## Security Constraints (Minimal)

### Required Validations:
1. **Slippage Protection**: Enforce max slippage (default 0.5%)
2. **Gas Limit**: Validate gas limits before execution
3. **Max Approval**: Never set infinite approval
4. **Signer Isolation**: Use configured signer, not dynamic

### Configuration:
```
# .env
RPC_URL=...
PRIVATE_KEY=...  # Optional, required for real execution
SLIPPAGE_MAX=0.5  # Max slippage percentage
GAS_LIMIT_MAX=500000  # Max gas limit
```

---

## Acceptance Criteria - Pending Verification

### Continuous Execution Test:
- [ ] 1000 query tool calls
- [ ] 100 real swaps (small amount on testnet)
- [ ] System no crash

### Per-Transaction Verification:
- [ ] Success: Returns real `tx_hash`
- [ ] Failure: Returns clear error reason

---

## Priority Ranking

| Priority | Tool                         | Effort | Impact |
| -------- | ---------------------------- | ------ | ------ |
| P0       | swap_tokens (real execution) | Medium | High   |
| P1       | news_search                  | Low    | Medium |
| P1       | onchain_transfer_monitor     | Medium | Medium |
| P1       | market_volume                | Medium | Medium |
| P2       | equity_price                 | Low    | Low    |

---

## Directory Structure Update

```
src/
├── domain/
│   ├── model/           # Add: News, Transfer, MarketData, Equity
│   ├── service/         # Add: NewsService, TransferService, MarketDataService, EquityService
│   └── repository/      # Add: TransferRepository, MarketDataRepository
├── infrastructure/
│   ├── ethereum.rs      # Add: get_transfers(), get_market_data()
│   ├── news_client.rs   # New: RSS/API client
│   └── market_client.rs # New: Market data API
└── interface/tools/
    ├── news.rs          # New
    ├── transfer.rs      # New
    ├── market_volume.rs # New
    └── equity.rs        # New
```

