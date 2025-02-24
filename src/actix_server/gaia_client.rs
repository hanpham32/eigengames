use openai_dive::v1::{
    api::Client,
    resources::chat::{
        ChatCompletionParametersBuilder, ChatCompletionParametersBuilderError,
        ChatCompletionResponse, ChatCompletionResponseFormat, ChatMessage, ChatMessageContent,
    },
};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum APIError {
    #[error("Request error: {0}")]
    ReqwestError(#[from] reqwest::Error),

    #[error("OpenAiDive error: {0}")]
    OpenAiDiveError(#[from] openai_dive::v1::error::APIError),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Chat completion error: {0}")]
    ChatCompletionError(#[from] ChatCompletionParametersBuilderError),

    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("IO error: {0}")]
    IOError(String),
}

pub struct GaiaNodeClient {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
}

impl GaiaNodeClient {
    pub fn new(base_url: String, api_key: String, model: String) -> Self {
        Self {
            base_url,
            api_key,
            model: model,
        }
    }

    pub fn chat() -> Result<ChatCompletionResponse, APIError> {
        /// TODO
    }
}
