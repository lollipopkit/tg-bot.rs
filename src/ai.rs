use anyhow::{anyhow, Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

use crate::consts::AI_PROMPT;

pub struct OpenAI {
    client: Client,
    api_key: String,
    model: String,
    api_url: String,
}

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
}

// Enhanced response models to handle error cases
#[derive(Deserialize, Debug)]
struct ChatCompletionResponse {
    choices: Option<Vec<ChatCompletionChoice>>,
    error: Option<OpenAIError>,
}

#[derive(Deserialize, Debug)]
struct ChatCompletionChoice {
    message: ChatResponseMessage,
}

#[derive(Deserialize, Debug)]
struct ChatResponseMessage {
    content: String,
}

#[derive(Deserialize, Debug)]
struct OpenAIError {
    message: String,
    // #[serde(rename = "type")]
    // error_type: String,
    // code: Option<String>,
    // param: Option<String>,
}

impl OpenAI {
    pub fn new() -> Result<Self> {
        let api_key =
            env::var("OPENAI_API_KEY").context("OPENAI_API_KEY environment variable not set")?;

        let model = env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-3.5-turbo".to_string());

        // Get custom base URL from environment or use default
        let api_base =
            env::var("OPENAI_API_BASE").unwrap_or_else(|_| "https://api.openai.com".to_string());
        let api_url = if api_base.ends_with("/v1/chat/completions") {
            api_base
        } else {
            format!("{}/v1/chat/completions", api_base.trim_end_matches('/'))
        };

        log::info!("Using OpenAI API URL: {}", api_url);

        Ok(Self {
            client: Client::new(),
            api_key,
            model,
            api_url,
        })
    }

    pub async fn generate_response(
        &self, 
        messages: Vec<(String, String)>,
        prompter: String,
    ) -> Result<String> {
        let mut msgs = vec![ChatMessage {
            role: "system".to_string(),
            content: AI_PROMPT.to_string(),
        }];

        let fmted_msgs: String = messages
            .iter()
            .map(|(user, content)| format!("[{}]: [{}]", user, content))
            .collect::<Vec<String>>()
            .join("\n");
        msgs.push(ChatMessage {
            role: "user".to_string(),
            content: fmted_msgs.replace("[USER_ID_LOCATOR]", &prompter),
        });

        let request = ChatCompletionRequest {
            model: self.model.clone(),
            messages: msgs,
        };

        // Send request
        let response = self
            .client
            .post(&self.api_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        // Check HTTP status first
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(anyhow!(
                "OpenAI API returned error status {}: {}",
                status,
                error_text
            ));
        }

        // Debug response body
        let response_body = response.text().await?;
        log::debug!("OpenAI raw response: {}", response_body);

        // Parse response
        let parsed_response: ChatCompletionResponse =
            serde_json::from_str(&response_body).context("Failed to parse OpenAI response")?;

        // Check for API-level error
        if let Some(error) = parsed_response.error {
            log::error!("OpenAI API error: {:?}", error);
            return Err(anyhow!("OpenAI API error: {}", error.message));
        }

        // Extract content
        match parsed_response.choices {
            Some(choices) if !choices.is_empty() => Ok(choices[0].message.content.clone()),
            _ => Err(anyhow!("No valid choices in OpenAI response")),
        }
    }
}
