use std::env;
use std::error::Error;

use serde::Deserialize;
use ureq::serde_json::Value;

#[derive(Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: Option<String>,
}

#[derive(Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<Choice>,
}

/// Sends a chat completion request to the OpenAI API and retrieves the response.
///
/// # Arguments
/// - `payload`: A `serde_json::Value` representing the request payload, typically containing
///   the model, messages, temperature, and other parameters.
///
/// # Returns
/// - `Ok(String)`: The content of the first choice in the response.
/// - `Err(Box<dyn Error>)`: An error if the request fails, the response cannot be parsed,
///   or no valid content is returned.
///
/// # Errors
/// - Returns an error if the `OPENAI_API_KEY` environment variable is not set.
/// - Returns an error if the request fails or the response cannot be deserialized.
/// - Returns an error if the response does not contain a valid choice or message content.
///
/// # HTTP Client Choice
/// We use `ureq` over `reqwest` due to the simplicity of the building/integration process.
/// Integrating `reqwest` required additional configuration that I preferred to avoid.
///
pub fn completion(payload: Value) -> Result<String, Box<dyn Error>> {
    let api_key = match env::var("OPENAI_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            return Err("OPENAI_API_KEY is missing. Set the environment variable (OPENAI_API_KEY=...) before usage to enable AI meta-commands.".into());
        }
    };

    let response = ureq::post("https://api.openai.com/v1/chat/completions")
        .set("Authorization", &format!("Bearer {}", api_key))
        .set("Content-Type", "application/json")
        .send_json(payload)?;

    let response_data: ChatCompletionResponse = response.into_json()?;

    let content = match &response_data.choices.first() {
        Some(choice) => match &choice.message.content {
            Some(content) => content.clone(),
            None => return Err("No content in response".into()),
        },
        None => return Err("No choices in response".into()),
    };

    Ok(content)
}
