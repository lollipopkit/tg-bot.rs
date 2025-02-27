use anyhow::{anyhow, Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{env, sync::Arc};

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
    pub fn new() -> Result<Arc<Self>> {
        let api_key =
            env::var("OPENAI_API_KEY").context("OPENAI_API_KEY environment variable not set")?;

        let model = env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string());

        // Get custom base URL from environment or use default
        let api_base =
            env::var("OPENAI_API_BASE").unwrap_or_else(|_| "https://api.openai.com".to_string());
        let api_url = if api_base.ends_with("/v1/chat/completions") {
            api_base
        } else {
            format!("{}/v1/chat/completions", api_base.trim_end_matches('/'))
        };

        log::info!("Using OpenAI API URL: {}", api_url);

        Ok(Arc::new(Self {
            client: Client::new(),
            api_key,
            model,
            api_url,
        }))
    }

    pub async fn generate_response(&self, messages: Vec<(String, String)>) -> Result<String> {
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
            content: fmted_msgs,
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


// OpenAI related constants

const AI_PROMPT: &str = r#"
你是一个机器人，ID 为 @lollipopkit_bot，名称是 lpktb。
你现在在一个群聊、私聊里。
请你扮演一个很会聊天的人，很有意思，很有趣。不要死气沉沉，太过于正式。

聊天记录将以如下格式给你：
```
[USER_ID]: [MESSAGE]
[THIS_IS_SEPERATOR_LINE]
[USER_ID]: [MESSAGE]
...
```

你只回复与你有关的消息。
如果历史记录与用户请求无关，请忽略它。
但是其他消息你可能也需要注意，因为可能是有用的上下文。

你的回答的语言，是最后一条消息的语言。

接下来聊天的内容：
"#;