use actix_web::web::JsonConfig;
use actix_web::error::{ErrorBadRequest, JsonPayloadError};
use serde_json::error::Category;
use log::error;

pub fn json_config() -> JsonConfig {
    JsonConfig::default()
        .limit(4096)
        .content_type_required(true)
        .error_handler(|err, _req| {
            error!("JSON parsing error: {:?}", err);
            
            let user_message = match err {
                JsonPayloadError::Deserialize(err) => match err.classify() {
                    Category::Syntax => {
                        if err.to_string().contains("control character") {
                            "Invalid JSON: Unescaped special character (like backslash) detected"
                        } else if err.to_string().contains("unexpected end of input") {
                            "Invalid JSON: Unclosed string or missing quotes"
                        } else {
                            "Invalid JSON syntax"
                        }
                    },
                    Category::Data => "Invalid JSON data format",
                    Category::Eof => "Incomplete JSON data received",
                    _ => "Malformed JSON request",
                },
                JsonPayloadError::Overflow { .. } => "Request payload too large (max 4KB)",
                JsonPayloadError::ContentType => "Content-Type must be application/json",
                JsonPayloadError::Payload(_) => "Error reading request body",
                _ => "Bad request",
            };
            
            ErrorBadRequest(user_message)
        })
}



