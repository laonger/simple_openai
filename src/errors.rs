use std::io::Write;
use std::{
    env,
    error,
    fmt,
    fmt::Display,
    fs::File,
};
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OpenAIError {
    err: String
}
impl Display for OpenAIError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&*self.err, f)
    }
}
impl error::Error for OpenAIError {
}
impl OpenAIError {
    pub fn from_string(error_msg: String) -> Self {
        Self {
            err: error_msg
        }
    }
    pub fn from_str(error_msg: &str) -> Self {
        Self {
            err: error_msg.to_string()
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct ResponseErrorContent {
    pub message:String,
}

#[derive(Deserialize, Debug)]
pub struct OpenAIErrorResponse {
    pub error: ResponseErrorContent,
}

pub type OError = Box<dyn std::error::Error + Send + Sync>;

pub type OpenAIResult<T> 
    = std::result::Result<T, OError>;
