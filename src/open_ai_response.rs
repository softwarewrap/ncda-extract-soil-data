use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAIResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    pub choices: Vec<Choice>,
    usage: Usage,
    system_fingerprint: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Choice {
    index: u32,
    pub message: Message,
    logprobs: Option<serde_json::Value>,
    finish_reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}
