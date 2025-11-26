# Ethereum Trading MCP Server

An MCP server implemented in Rust that enables AI agents to interact with the Ethereum blockchain. It provides tools for querying balances, checking token prices, and simulating Uniswap token swaps.

## Features

- **`get_balance`**: Query ETH and ERC20 token balances.
- **`get_token_price`**: Get current token price in USDC (via Uniswap V3 Quoter).
- **`swap_tokens`**: Simulate Uniswap V3 swaps to estimate output and gas.

## Setup

### Prerequisites

- Rust (latest stable)
- An Ethereum RPC URL (e.g., from Alchemy or Infura)

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

### Running

```bash
cargo run
```

The server listens on Stdio for MCP JSON-RPC requests.

## Usage

### Example MCP Request (`get_balance`)

```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "get_balance",
    "arguments": {
      "address": "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"
    }
  },
  "id": 1
}
```

### Example MCP Request (`swap_tokens`)

```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "swap_tokens",
    "arguments": {
      "from_token": "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
      "to_token": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
      "amount": "1.0"
    }
  },
  "id": 2
}
```

## Limitations

- **Price Checking**: Currently relies on a direct Uniswap V3 pool (Token/USDC). Tokens without a direct USDC pool might fail to return a price.
- **Simulation**: `swap_tokens` simulates the transaction but does not execute it. It assumes a 0.3% fee tier.
- **Network**: Only Mainnet is fully supported with hardcoded contract addresses.

## Testing

Run unit tests:

```bash
cargo test
```
