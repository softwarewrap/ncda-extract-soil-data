use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAIRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub max_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: Vec<Content>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Content {
    pub r#type: String,
    #[serde(flatten)]
    pub data: ContentData,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ContentData {
    Text { text: String },
    ImageUrl { image_url: ImageUrl },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageUrl {
    pub url: String,
    pub detail: String,
}
