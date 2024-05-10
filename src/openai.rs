use std::io::Write;
use std::{
    env,
    error,
    fmt,
    fmt::Display,
};
use tempfile;
use std::collections::HashMap;


use tokio;

use hyper_util::rt::TokioIo;
use tokio::net::TcpStream;

use hyper::{
    body::{Buf, Bytes, Body},
    header,
    Request,
    http::status::StatusCode,
};
use hyper_tls::HttpsConnector;
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;


use http_body::Body as HttpBody;
use http_body_util::{
    Full,
    BodyExt,
};

use crate::utils;
use crate::errors::{
    OpenAIErrorResponse,
    OpenAIError,
    OpenAIResult,
};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
//#[serde(tag="role", content="content")]
pub enum RoleType {
    assistant,
    user,
    system,
}

//#[derive(Deserialize, Debug, Clone)]
//struct ResponseErrorContent {
//    message:String,
//}
//
//#[derive(Deserialize, Debug)]
//struct OpenAIErrorResponse {
//    error: ResponseErrorContent,
//}
//

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FuncResponse {
    pub name: String,
    //pub arguments: HashMap<String, String>,
    pub arguments: Value,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ResponseMessageUnit {
    pub role: RoleType,
    pub content: Option<String>,
    pub function_call: Option<FuncResponse>
}

#[derive(Deserialize, Debug, Clone)]
struct ResponseChoiseUnit {
    message:ResponseMessageUnit,
}

#[derive(Deserialize, Debug, Clone)]
struct ResponseTokenUnit {
    prompt_tokens: usize,
    completion_tokens: usize,
    total_tokens: usize
}

#[derive(Deserialize, Debug)]
struct OpenAIResponse {
    choices: Vec<ResponseChoiseUnit>,
    usage: ResponseTokenUnit
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RequestMessageUnit {
    pub role: RoleType,
    pub content: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct OpenAIRequest {
    model: String,
    messages: Vec<RequestMessageUnit>,
    functions: Option<Vec<FuncUnit>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct OpenAIImageRequest {
    prompt: String,
    n: i32,
    size: String
}


#[derive(Serialize, Deserialize, Debug)]
struct OpenAIImageResponseData {
    url: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct OpenAIImageResponse {
    created: i64,
    data: Vec<OpenAIImageResponseData>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FuncParamUnit {
    #[serde(rename = "type")]
    pub t: String,
    #[serde(rename = "enum")]
    pub e: Vec<String>,
    pub description: String,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
//pub struct FuncParams {
pub struct FuncParams {
    #[serde(rename = "type")]
    pub t: String,
    pub required: Vec<String>,
    pub properties: HashMap<String, FuncParamUnit>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FuncUnit {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub parameters: Option<FuncParams>,
}

pub type OError = Box<dyn std::error::Error + Send + Sync>;

pub type Result<T> 
    = std::result::Result<T, OError>;

pub async fn ask(
        messages: Vec<RequestMessageUnit>, functions: Option<Vec<FuncUnit>>
    ) -> Result<(ResponseMessageUnit, usize, usize)> {

    let uri = "https://api.openai.com/v1/chat/completions".to_string();

    let model = String::from("gpt-4-1106-preview");

    let openai_request = OpenAIRequest {
        model,
        messages,
        functions,
    };

    let r = serde_json::to_string(&openai_request)?;

    let mut res = utils::request(uri, r).await.unwrap();

    match res.status() {
        StatusCode::OK => {
            let body = res.collect().await?.aggregate();
            //println!("openai res body: {:?}", String::from_utf8(body.reader()));
            let json_raw: Value = match serde_json
                ::from_reader(body.reader()) {
                    Ok(j) => j,
                    Err(e) => {
                        return Err(Box::new(e))
                    }
            };
            let json: OpenAIResponse = match serde_json
                ::from_value(json_raw.clone()){
                    Ok(j) => j,
                    Err(e) => {
                        eprintln!("json parse Error: {:?}\n{:?}", e, json_raw);
                        return Err(Box::new(e))
                    }
                };
            //println!("openai res json, {:?}", json);
            return Ok((
                json.choices[0].clone().message,
                json.usage.prompt_tokens,
                json.usage.completion_tokens
            ));
        },
        StatusCode::BAD_REQUEST => {
            let body = res.collect().await?.aggregate();
            let error: OpenAIErrorResponse = serde_json::from_reader(body.reader())?;
            Err(Box::new(OpenAIError::from_string(error.error.message)))
        },
        _ => {
            eprintln!("Error res: {:?}", res);
            let body = res.collect().await?.aggregate();
            let error: OpenAIErrorResponse = serde_json::from_reader(body.reader())?;
            Err(Box::new(OpenAIError::from_string(error.error.message)))
        }
    }
}

pub async fn draw(prompt: String, n: i32, size: String) -> Result<String> {

    let uri = "https://api.openai.com/v1/images/generations".to_string();

    let s:&str = &size;

    match s {
        "1024x1024" => {},
        "512x512" => {},
        "256x256" => {},
        _ => {
            return Ok("size only support: 1024x1024, 512x512, 256x256".to_string())
        }
    }

    let openai_request = OpenAIImageRequest {
        prompt,
        n,
        size
    };

    let r = serde_json::to_string(&openai_request)?;
    let res = utils::request(uri, r).await.unwrap();

    match res.status() {
        StatusCode::OK => {
            let body = res.collect().await?.aggregate();
            let json: OpenAIImageResponse
                = serde_json::from_reader(body.reader())?;
            let mut result: Vec<String> = Vec::new();
            for i in json.data {
                result.push(i.url)
            }
            return Ok(result.join("\n"))
        },
        StatusCode::BAD_REQUEST => {
            let body = res.collect().await?.aggregate();
            let error: OpenAIErrorResponse = serde_json::from_reader(body.reader())?;
            Err(Box::new(OpenAIError::from_string(error.error.message)))
        },
        _ => {
            eprintln!("Error res: {:?}", res);
            let body = res.collect().await?.aggregate();
            let error: OpenAIErrorResponse = serde_json::from_reader(body.reader())?;
            Err(Box::new(OpenAIError::from_string(error.error.message)))
        }
    }
}


// curl https://api.openai.com/v1/audio/speech \
//     -H "Authorization: Bearer $OPENAI_API_KEY" \
//     -H "Content-Type: application/json" \
//     -d '{
//       "model": "tts-1",
//       "input": "The quick brown fox jumped over the lazy dog.",
//       "voice": "alloy"
//     }' \
//     --output speech.mp3
//

pub enum OpenAISpeedVoice {
    Alloy    ,
    Echo     ,
    Fable    ,
    Onyx     ,
    Nova     ,
    Shimmer  ,
}

#[derive(Serialize, Deserialize, Debug)]
struct OpenAISpeedRequest {
    model: String,
    input: String,
    voice: String,
}


fn get_voice (voice: OpenAISpeedVoice) -> String {
    match voice {
        OpenAISpeedVoice::Alloy    => "alloy".to_string()   ,
        OpenAISpeedVoice::Echo     => "echo".to_string()    ,
        OpenAISpeedVoice::Fable    => "fable".to_string()   ,
        OpenAISpeedVoice::Onyx     => "onyx".to_string()    ,
        OpenAISpeedVoice::Nova     => "nova".to_string()    ,
        OpenAISpeedVoice::Shimmer  => "shimmer".to_string() ,
        
    }
}

pub async fn speak(
        text: String,
        voice: OpenAISpeedVoice,
        mut output: &tempfile::NamedTempFile
    ) -> Result<tempfile::NamedTempFile> {

    let url = "https://api.openai.com/v1/audio/speech".to_string();

    let model = String::from("tts-1");
    
    let openai_request = OpenAISpeedRequest {
        model,
        input: text,
        voice: get_voice(voice)
    };

    let r = serde_json::to_string(&openai_request)?;

    let mut res = utils::request(url, r).await.unwrap();

    match res.status() {
        StatusCode::OK => {
            //let body = hyper::body::aggregate(res).await?;
            //println!("openai res body: {:?}", String::from_utf8(body.reader()));
            while let Some(next) = res.frame().await {
                let frame = next?;
                if let Some(chunk) = frame.data_ref(){
                    let _ = output
                        .write_all(chunk)
                        .map_err(|e|{
                            eprintln!("download speed ERROR: {:?}", e)
                        });
                }
            }
            return Ok(output);
        },
        StatusCode::BAD_REQUEST => {
            let body = res.collect().await?.aggregate();
            let error: OpenAIErrorResponse = serde_json::from_reader(body.reader())?;
            Err(Box::new(OpenAIError::from_string(error.error.message)))
        },
        _ => {
            eprintln!("Error res: {:?}", res);
            let body = res.collect().await?.aggregate();
            let error: OpenAIErrorResponse = serde_json::from_reader(body.reader())?;
            Err(Box::new(OpenAIError::from_string(error.error.message)))
        }
    }
}

#[tokio::test]
async fn function_name_test() {
    let model = String::from("gpt-3.5-turbo");
    let openai_request = OpenAIRequest {
        model,
        messages: vec![
            RequestMessageUnit {
                role: RoleType::user,
                content: None,
            },
        ],
        functions: Some(vec![
            FuncUnit{
                name: "set_role".to_string(),
                description: "if users want to clear the bot's role set".to_string(),
                parameters: None
            }
        ])
    };

    eprintln!("{}", serde_json::to_string(&openai_request).unwrap());
    
}

