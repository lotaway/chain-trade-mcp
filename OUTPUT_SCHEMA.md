# MCP 工具输出格式文档

## 概述

现在所有三个工具都提供了完整的输入和输出格式定义（schema），这样 AI 模型就能清楚地知道：
1. 如何调用工具（输入格式）
2. 会得到什么样的响应（输出格式）

## 工具列表响应示例

当 AI 模型请求 `tools/list` 时，会收到以下格式的响应：

```json
{
  "tools": [
    {
      "name": "get_balance",
      "description": "Get the balance of ETH or an ERC20 token for a specific address",
      "inputSchema": {
        "type": "object",
        "properties": {
          "address": {
            "type": "string",
            "description": "The wallet address to check"
          },
          "token_address": {
            "type": "string",
            "description": "Optional ERC20 token contract address. If omitted, returns ETH balance."
          }
        },
        "required": ["address"]
      },
      "outputSchema": {
        "type": "object",
        "description": "Balance information wrapped in MCP content format",
        "properties": {
          "content": {
            "type": "array",
            "items": {
              "type": "object",
              "properties": {
                "type": {"type": "string", "const": "text"},
                "text": {
                  "type": "string",
                  "description": "JSON string containing balance data with fields: token (optional object with address/symbol/decimals), amount (wei/smallest unit as string), formatted (human-readable amount as string)"
                }
              }
            }
          }
        }
      }
    },
    {
      "name": "get_token_price",
      "description": "Get the current price of a token in USDC",
      "inputSchema": {
        "type": "object",
        "properties": {
          "token_address": {
            "type": "string",
            "description": "The token contract address"
          }
        },
        "required": ["token_address"]
      },
      "outputSchema": {
        "type": "object",
        "description": "Token price information in MCP content format",
        "properties": {
          "content": {
            "type": "array",
            "items": {
              "type": "object",
              "properties": {
                "type": {"type": "string", "const": "text"},
                "text": {
                  "type": "string",
                  "description": "JSON string containing price data with fields: token_address (string), price (string in USDC)"
                }
              }
            }
          }
        }
      }
    },
    {
      "name": "swap_tokens",
      "description": "Simulate a token swap on Uniswap V3 to get estimated output and gas usage",
      "inputSchema": {
        "type": "object",
        "properties": {
          "from_token": {
            "type": "string",
            "description": "Address of the token to sell"
          },
          "to_token": {
            "type": "string",
            "description": "Address of the token to buy"
          },
          "amount": {
            "type": "string",
            "description": "Amount of from_token to sell (human readable, e.g. 1.5)"
          },
          "slippage": {
            "type": "number",
            "description": "Slippage tolerance (e.g. 0.005 for 0.5%)"
          }
        },
        "required": ["from_token", "to_token", "amount"]
      },
      "outputSchema": {
        "type": "object",
        "description": "Swap simulation quote in MCP content format",
        "properties": {
          "content": {
            "type": "array",
            "items": {
              "type": "object",
              "properties": {
                "type": {"type": "string", "const": "text"},
                "text": {
                  "type": "string",
                  "description": "JSON string containing swap quote with fields: from_token (address), to_token (address), input_amount (string), estimated_output (string with slippage info), gas_estimate (string), simulation_success (boolean), error_message (optional string)"
                }
              }
            }
          }
        }
      }
    }
  ]
}
```

## 实际响应示例

### get_balance (ETH)

**请求：**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "get_balance",
    "arguments": {
      "address": "0x8C864D0c8E476Bf9eb9d620C10E1296fb0E2F940"
    }
  },
  "id": 1
}
```

**响应：**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\n  \"token\": null,\n  \"amount\": \"1\",\n  \"formatted\": \"0.000000000000000001\"\n}"
      }
    ]
  },
  "id": 1
}
```

### get_balance (ERC20)

**请求：**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "get_balance",
    "arguments": {
      "address": "0x8C864D0c8E476Bf9eb9d620C10E1296fb0E2F940",
      "token_address": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
    }
  },
  "id": 1
}
```

**响应：**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\n  \"token\": {\n    \"address\": \"0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48\",\n    \"symbol\": \"USDC\",\n    \"decimals\": 6\n  },\n  \"amount\": \"1000000\",\n  \"formatted\": \"1.0\"\n}"
      }
    ]
  },
  "id": 1
}
```

### get_token_price

**请求：**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "get_token_price",
    "arguments": {
      "token_address": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
    }
  },
  "id": 1
}
```

**响应：**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\n  \"token_address\": \"0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48\",\n  \"price\": \"1.0\"\n}"
      }
    ]
  },
  "id": 1
}
```

### swap_tokens

**请求：**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "swap_tokens",
    "arguments": {
      "from_token": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
      "to_token": "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
      "amount": "1000000",
      "slippage": 0.01
    }
  },
  "id": 1
}
```

**响应（成功）：**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\n  \"from_token\": \"0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48\",\n  \"to_token\": \"0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2\",\n  \"input_amount\": \"1000000\",\n  \"estimated_output\": \"0.000123 (min: 0.000122 with 1% slippage)\",\n  \"gas_estimate\": \"Unknown\",\n  \"simulation_success\": true,\n  \"error_message\": null\n}"
      }
    ]
  },
  "id": 1
}
```

**响应（失败 - STF 错误）：**
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32000,
    "message": "server returned an error response: error code 3: execution reverted: STF",
    "data": null
  },
  "id": 1
}
```

## AI 模型的好处

有了 `outputSchema`，AI 模型现在可以：

1. **理解响应结构**：知道响应是 MCP 格式，需要从 `content[0].text` 中提取 JSON
2. **解析数据字段**：知道每个工具返回哪些字段及其含义
3. **处理不同情况**：
   - `get_balance` 返回的 `token` 字段可能为 `null`（ETH）或对象（ERC20）
   - `swap_tokens` 可能成功返回报价，也可能失败返回错误
4. **生成更好的提示**：可以告诉用户期望得到什么样的数据

## 实现细节

### Tool Trait

```rust
pub trait Tool: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn input_schema(&self) -> Value;
    
    // 新增：输出格式定义
    fn output_schema(&self) -> Value {
        // 默认实现：标准 MCP 格式
    }
    
    async fn execute(&self, client: &EthereumClient, args: &Value) -> Result<Value, String>;
}
```

### 每个工具的实现

每个工具都覆盖了 `output_schema()` 方法，提供具体的输出格式说明。

## 总结

✅ **问题已解决**：所有三个工具现在都有完整的输出格式定义

✅ **AI 友好**：AI 模型可以清楚地知道会收到什么格式的数据

✅ **向后兼容**：默认实现确保即使工具没有自定义 `output_schema`，也会返回标准 MCP 格式说明

✅ **测试通过**：所有测试仍然正常工作
