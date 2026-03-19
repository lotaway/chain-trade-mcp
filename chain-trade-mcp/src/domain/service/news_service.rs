use crate::domain::model::news::{NewsArticle, NewsSearchResult};
use async_trait::async_trait;
use reqwest;
use std::result::Result;

pub struct NewsService;

impl NewsService {
    pub fn new() -> Self {
        Self
    }

    async fn fetch_rss_news(&self, query: &str, limit: u32) -> Result<NewsSearchResult, String> {
        let rss_feeds = [
            "https://cryptopanic.com/news/rss",
            "https://news.google.com/rss/search?q=cryptocurrency",
        ];

        let client = reqwest::Client::new();
        let mut articles = Vec::new();

        for feed_url in &rss_feeds {
            if articles.len() >= limit as usize {
                break;
            }

            match self.fetch_rss_feed(&client, feed_url, query).await {
                Ok(mut feed_articles) => {
                    articles.append(&mut feed_articles);
                }
                Err(e) => {
                    tracing::warn!("Failed to fetch RSS feed {}: {}", feed_url, e);
                }
            }
        }

        articles.truncate(limit as usize);
        articles.sort_by(|a, b| b.pub_date.cmp(&a.pub_date));

        let total_count = articles.len();

        Ok(NewsSearchResult {
            query: query.to_string(),
            articles,
            total_count,
        })
    }

    async fn fetch_rss_feed(
        &self,
        client: &reqwest::Client,
        feed_url: &str,
        _query: &str,
    ) -> Result<Vec<NewsArticle>, String> {
        let response = client
            .get(feed_url)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let content = response.text().await.map_err(|e| e.to_string())?;

        let articles = self.parse_rss_content(&content, feed_url)?;

        Ok(articles)
    }

    fn parse_rss_content(&self, content: &str, source: &str) -> Result<Vec<NewsArticle>, String> {
        let mut articles = Vec::new();

        let start_indices: Vec<usize> = content.match_indices("<item>").map(|(i, _)| i).collect();

        for start in start_indices.iter().take(5) {
            let item_end = content[*start..]
                .find("</item>")
                .map(|end| start + end)
                .unwrap_or(content.len());

            let item_content = &content[*start..item_end];

            let title = self.extract_rss_field(item_content, "<title>", "</title>");
            let link = self.extract_rss_field(item_content, "<link>", "</link>");
            let description =
                self.extract_rss_field(item_content, "<description>", "</description>");
            let pub_date = self.extract_rss_field(item_content, "<pubDate>", "</pubDate>");

            if !title.is_empty() && !link.is_empty() {
                articles.push(NewsArticle {
                    title: self.clean_html(&title),
                    link,
                    description: self.clean_html(&description),
                    pub_date,
                    source: source.to_string(),
                });
            }
        }

        Ok(articles)
    }

    fn extract_rss_field(&self, content: &str, start_tag: &str, end_tag: &str) -> String {
        if let Some(start) = content.find(start_tag) {
            let content_start = start + start_tag.len();
            if let Some(end) = content[content_start..].find(end_tag) {
                return content[content_start..content_start + end].to_string();
            }
        }
        String::new()
    }

    fn clean_html(&self, text: &str) -> String {
        let mut result = text.to_string();
        result = result.replace("<![CDATA[", "");
        result = result.replace("]]>", "");
        result = result.replace("<p>", "");
        result = result.replace("</p>", "");
        result = result.replace("<br />", "");
        result = result.replace("\n", " ");
        result = result.trim().to_string();
        result
    }

    async fn fetch_cryptopanic_news(
        &self,
        query: &str,
        limit: u32,
    ) -> Result<NewsSearchResult, String> {
        let url = format!(
            "https://cryptopanic.com/api/v1/posts/?auth_token=demo&search={}",
            query
        );

        let client = reqwest::Client::new();
        let response = client.get(&url).send().await.map_err(|e| e.to_string())?;

        let json: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;

        let mut articles = Vec::new();

        if let Some(results) = json.get("results").and_then(|v| v.as_array()) {
            for item in results.iter().take(limit as usize) {
                let title = item
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let link = item
                    .get("url")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let pub_date = item
                    .get("created_at")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                if !title.is_empty() {
                    articles.push(NewsArticle {
                        title,
                        link,
                        description: String::new(),
                        pub_date,
                        source: "cryptopanic".to_string(),
                    });
                }
            }
        }

        let total_count = articles.len();

        Ok(NewsSearchResult {
            query: query.to_string(),
            articles,
            total_count,
        })
    }
}

#[async_trait]
pub trait NewsRepository: Send + Sync {
    async fn search_news(
        &self,
        query: &str,
        limit: Option<u32>,
        source: Option<&str>,
    ) -> Result<NewsSearchResult, String>;
}

#[async_trait]
impl NewsRepository for NewsService {
    async fn search_news(
        &self,
        query: &str,
        limit: Option<u32>,
        source: Option<&str>,
    ) -> Result<NewsSearchResult, String> {
        let limit = limit.unwrap_or(10).min(20);

        match source.unwrap_or("rss") {
            "rss" => self.fetch_rss_news(query, limit).await,
            "cryptopanic" => self.fetch_cryptopanic_news(query, limit).await,
            _ => self.fetch_rss_news(query, limit).await,
        }
    }
}
