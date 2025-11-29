# Ethereum Trading MCP Server

An MCP (Model Context Protocol) server implemented in Rust using the official `rmcp` SDK. Enables AI agents to interact with the Ethereum blockchain for querying balances, checking token prices, and simulating Uniswap token swaps.

## Features

- **`get_balance`**: Query ETH and ERC20 token balances.
- **`get_token_price`**: Get current token price in USDC (via Uniswap V3 Quoter).
- **`swap_tokens`**: Simulate Uniswap V3 swaps to estimate output and gas.

## Setup

### Prerequisites

- Rust (latest stable)
- An Ethereum RPC URL, already set a public free RPC URL inside the `src/config/mod.rs`, but a paid RPC URL will be better (e.g., from Alchemy or Infura)

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

### Building

Build the release version:

```bash
cargo build --release
```

The binary will be located at `target/release/chain-trade-mcp`.

## Usage with MCP Clients

### Configuration Example (Claude Desktop, Cline, etc.)

Add the following to your MCP client configuration file:

**For Claude Desktop** (`~/Library/Application Support/Claude/claude_desktop_config.json` on macOS):

```json
{
  "mcpServers": {
    "chain-trade": {
      "command": "/opt/chain-trade-mcp/chain-trade-mcp",
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

**Important**: Replace the `command` path with the absolute path to your compiled binary.

### Available Tools

Once configured, AI assistants can use the following tools:

1. **get_balance** - Query ETH or ERC20 token balance
   ```json
   {
     "address": "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
     "token_address": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"  // Optional
   }
   ```

2. **get_token_price** - Get token price in USDC
   ```json
   {
     "token": "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"
   }
   ```

3. **swap_tokens** - Simulate token swap (does not execute)
   ```json
   {
     "from_token": "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
     "to_token": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
     "amount": "1.0",
     "slippage": 0.5  // Optional, defaults to config value
   }
   ```

## Running Standalone

You can also run the server directly for testing:

```bash
cargo run
```

The server listens on stdin/stdout for MCP JSON-RPC requests.

### Example MCP Request

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

## Limitations

- **Price Checking**: Currently relies on a direct Uniswap V3 pool (Token/USDC). Tokens without a direct USDC pool might fail to return a price.
- **Simulation**: `swap_tokens` simulates the transaction but does not execute it. It assumes a 0.3% fee tier.
- **Network**: Only Mainnet is fully supported with hardcoded contract addresses.

## Testing

Run unit tests:

```bash
cargo test
```

All 43 tests should pass.
