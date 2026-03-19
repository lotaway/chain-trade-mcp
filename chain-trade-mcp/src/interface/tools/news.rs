use crate::domain::service::news_service::{NewsRepository, NewsService};
use crate::infrastructure::ethereum::EthereumClient;
use crate::interface::tools::tool_trait::Tool;
use async_trait::async_trait;
use serde_json::{json, Value};

pub struct NewsTool;

#[async_trait]
impl Tool for NewsTool {
    fn name(&self) -> &'static str {
        "news_search"
    }

    fn description(&self) -> &'static str {
        "Search for cryptocurrency news from RSS feeds and public APIs. Returns raw articles without summarization."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": { "type": "string", "description": "Search query (e.g., 'ethereum', 'bitcoin')" },
                "limit": { "type": "integer", "description": "Maximum number of articles to return (default: 10, max: 20)" },
                "source": { "type": "string", "description": "News source: 'rss' or 'cryptopanic' (default: rss)" }
            },
            "required": ["query"]
        })
    }

    fn output_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "content": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "type": {"type": "string", "const": "text"},
                            "text": {"type": "string"}
                        }
                    }
                }
            }
        })
    }

    async fn execute(&self, _client: &EthereumClient, args: &Value) -> Result<Value, String> {
        let query = args
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or("Missing query")?;
        let limit = args.get("limit").and_then(|v| v.as_u64()).map(|v| v as u32);
        let source = args.get("source").and_then(|v| v.as_str());

        let news_service = NewsService::new();
        let result = news_service
            .search_news(query, limit, source)
            .await
            .map_err(|e| e.to_string())?;

        Ok(json!({
            "content": [
                {
                    "type": "text",
                    "text": serde_json::to_string_pretty(&result).unwrap()
                }
            ]
        }))
    }
}
