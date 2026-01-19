# Web3 AI Quant Agent – MCP Tool Spec

Provides **real, executable, and auditable** on-chain and market capabilities for Web3 quantitative agents, used for automated trading and risk control.


## Available Tools

### 🔴 P0 · Must-Have Tools (Real Execution)

| Tool              | Function                 | Status | Implementation                   |
| ----------------- | ------------------------ | ------ | -------------------------------- |
| `get_balance`     | Query on-chain balance   | ✅ Done | `src/interface/tools/balance.rs` |
| `get_token_price` | Real market price query  | ✅ Done | `src/interface/tools/price.rs`   |
| `swap_tokens`     | Real DEX trade execution | ✅ Done | `src/interface/tools/swap.rs`    |

#### swap_tokens Mandatory Requirements

```
✅ Must specify: dex / router
✅ Must specify: slippage
✅ Must specify: max_spend or min_receive
❌ Failure is final, no auto-retry
```

**Implementation Details**:
- `src/infrastructure/swap_executor.rs` - Real Swap Execution Engine
  - Private key signature verification
  - Slippage protection (max_slippage)
  - Gas limit verification
  - Balance check
  - Approval management
  - Returns real tx_hash

### 🟡 P1 · Recommended Tools (Real Data)

| Tool                       | Function                    | Status    | Description                        |
| -------------------------- | --------------------------- | --------- | ---------------------------------- |
| `news_search`              | Real news source retrieval  | ✅ Done    | RSS / CryptoPanic API              |
| `onchain_transfer_monitor` | On-chain transfer query     | 📋 Pending | Monitor abnormal fund flows        |
| `market_volume`            | Real volume / price changes | 📋 Pending | Liquidity and volatility filtering |

### 🔵 P2 · Optional Tools

| Tool           | Function                  | Status    |
| -------------- | ------------------------- | --------- |
| `equity_price` | Real stock / index prices | 📋 Pending |

---

## System Invariants

✅ These rules must always hold:

1. **Any execution tool** produces at most **one** on-chain transaction
2. Tools must not implicitly modify transaction parameters
3. Same input must not lead to non-deterministic behavior
4. **Query failure ≠ Execution failure**, strictly separated

### Security Constraints

Operations that agents CANNOT perform:

| ❌ Prohibited Operation       | Reason                            |
| ---------------------------- | --------------------------------- |
| Modify gas strategy limits   | Prevent unexpected high fees      |
| Bypass slippage              | Prevent malicious slippage losses |
| Initiate unlimited approvals | Prevent token theft               |

## Configuration

### Environment Variables

```env
# .env
RPC_URL=...              # Ethereum RPC URL
PRIVATE_KEY=...          # Private key (for signing execution)
PORT=3000
RUST_LOG=info
MAX_SLIPPAGE=0.05        # Max slippage limit
MAX_GAS_LIMIT=500000     # Max gas limit
```

### Configuration Options

| Config                   | Description                                |
| ------------------------ | ------------------------------------------ |
| `RPC_URL`                | Ethereum RPC endpoint                      |
| `PRIVATE_KEY`            | Private key (optional, for real execution) |
| `MAX_SLIPPAGE`           | Max slippage agent can set                 |
| `MAX_GAS_LIMIT`          | Max gas limit agent can set                |
| `UNISWAP_ROUTER_ADDRESS` | Uniswap V3 Router address                  |
| `UNISWAP_QUOTER_ADDRESS` | Uniswap V3 Quoter address                  |

---

## Setup

### Prerequisites

- Rust (latest stable)
- - An Ethereum RPC URL. A public free RPC URL is already configured in `src/config/mod.rs`, but a paid RPC URL is recommended for better performance (e.g., from Alchemy or Infura)

### Configuration

1. Copy `.env` template:
   ```bash
   cp .env.example .env
   ```
   *(Note: Create `.env` manually if not present)*

2. Edit `.env`:
   ```env
   RPC_URL=https://eth-mainnet.g.alchemy.com/v2/your-api-key
   PRIVATE_KEY=... (Optional, for signing if needed, currently used for simulation context)
   PORT=3000
   RUST_LOG=info
   ```

### Build

```bash
cargo build --release
```

Binary is located at `target/release/chain-trade-mcp`.

### Run

```bash
cargo run
```

Server listens for MCP JSON-RPC requests on stdin/stdout.

---

## MCP Client Configuration

### Claude Desktop

Add to `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "chain-trade": {
      "command": "/path/to/chain-trade-mcp",
      "type": "stdio",
      "timeout": 60,
      "env": {
        "RPC_URL": "https://eth-mainnet.g.alchemy.com/v2/YOUR_API_KEY",
        "RUST_LOG": "info"
      }
    }
  }
}
```

---

## Directory Structure

```
src/
├── domain/
│   ├── model/
│   │   ├── balance.rs
│   │   ├── news.rs
│   │   ├── swap_quote.rs
│   │   └── token.rs
│   ├── repository/
│   │   ├── balance_repository.rs
│   │   ├── price_repository.rs
│   │   └── swap_repository.rs
│   └── service/
│       ├── balance_service.rs
│       ├── news_service.rs
│       ├── price_service.rs
│       └── swap_service.rs
├── infrastructure/
│   ├── cache.rs
│   ├── ethereum.rs              // get_signer, get_config, get_provider
│   ├── notification.rs
│   ├── rpc/
│   │   ├── connection_pool.rs
│   │   └── rate_limiter.rs
│   └── swap_executor.rs         // Real swap execution
├── interface/
│   ├── mcp.rs                   // MCP protocol handler
│   └── tools/
│       ├── balance.rs           // get_balance
│       ├── news.rs              // news_search
│       ├── price.rs             // get_token_price
│       ├── swap.rs              // swap_tokens (real execution support)
│       └── tool_trait.rs
└── config/
    └── mod.rs                   // max_slippage, max_gas_limit
```

---

## Testing

```bash
cargo test
```

---

## Limitations

- **Price Query**: Relies on Uniswap V3 pool (Token/USDC), tokens without direct USDC pool may fail
- **Network**: Currently only supports mainnet
- **News Search**: Supports RSS and CryptoPanic API