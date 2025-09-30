use std::borrow::Cow;

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub async fn send_request(
    prompt: impl Into<Cow<'static, str>>,
    model: impl Into<Cow<'static, str>>,
) -> Result<impl futures_util::Stream<Item = Result<GenerationResponse>>> {
    let r = reqwest::Client::new();
    Ok(crate::bytes_line_stream::lines(
        r.post("http://127.0.0.1:11434/api/generate")
            .json(&GenerationRequest {
                model: model.into(),
                prompt: prompt.into(),
            })
            .send()
            .await
            .unwrap()
            .bytes_stream(),
    )
    .map(|d| {
        let bytes = d?;
        Ok(serde_json::from_slice(&bytes)?)
    }))
}

#[derive(Debug, Clone, Serialize)]
pub struct GenerationRequest<'a> {
    pub model: Cow<'a, str>,
    pub prompt: Cow<'a, str>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationResponse {
    /// The name of the model used for the completion.
    pub model: String,
    /// The creation time of the completion, in such format: `2023-08-04T08:52:19.385406455-07:00`.
    pub created_at: String,
    /// The response of the completion. This can be the entire completion or only a token if the completion is streaming.
    pub response: String,
    /// Whether the completion is done. If the completion is streaming, this will be false until the last response.
    pub done: bool,
    /// An encoding of the conversation used in this response, this can be sent in the next request to keep a conversational memory
    /// pub context: Option<GenerationContext>,
    /// Time spent generating the response
    pub total_duration: Option<u64>,
    /// Time spent in nanoseconds loading the model
    pub load_duration: Option<u64>,
    /// Number of tokens in the prompt
    pub prompt_eval_count: Option<u64>,
    /// Time spent in nanoseconds evaluating the prompt
    pub prompt_eval_duration: Option<u64>,
    /// Number of tokens in the response
    pub eval_count: Option<u64>,
    /// Time spent in nanoseconds generating the response
    pub eval_duration: Option<u64>,
    /// Contains the text that was inside thinking tags in the original model output when ChatMessageRequest.Think is enabled.
    pub thinking: Option<String>,
}

#[cfg(test)]
mod tests {
    use futures_util::TryStreamExt;

    use super::*;

    #[tokio::test]
    async fn send_ollama_request() {
        let prompt = "Tell me a story about the rust programming language.";
        let mut stream = std::pin::pin!(send_request(prompt, "qwen3:30B").await.unwrap());
        while let Some(x) = stream.try_next().await.unwrap() {
            eprintln!("Got: {}", x.response);
        }
    }
}
