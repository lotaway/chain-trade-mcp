use crate::config::AIQuantSettings;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct AIClient {
    settings: AIQuantSettings,
    client: reqwest::Client,
}

#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<Message>,
}

#[derive(Debug, Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: MessageResponse,
}

#[derive(Debug, Deserialize)]
struct MessageResponse {
    content: String,
}

impl AIClient {
    pub fn new(settings: AIQuantSettings) -> Self {
        Self {
            settings,
            client: reqwest::Client::new(),
        }
    }

    pub async fn get_trade_signal(&self, market_data: &str) -> Result<String> {
        if !self.settings.enabled {
            return Ok("AI Trading is disabled".to_string());
        }

        let request = ChatCompletionRequest {
            model: self.settings.model_name.clone(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: "You are a quantitative trading assistant. Analyze the market data and provide a trade signal.".to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: market_data.to_string(),
                },
            ],
        };

        let response = self.client
            .post(&self.settings.provider_url)
            .header("Authorization", format!("Bearer {}", self.settings.api_key))
            .json(&request)
            .send()
            .await?;

        let response_body: ChatCompletionResponse = response.json().await?;
        
        if let Some(choice) = response_body.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            Ok("No signal generated".to_string())
        }
    }
}
