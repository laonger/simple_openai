
use std::{
    env,
    error,
    fmt,
    fmt::Display,
};
use std::collections::HashMap;

use http::status::StatusCode;

use hyper::{body::Buf, header, Body, Client, Request};
use hyper_tls::HttpsConnector;
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
//#[serde(tag="role", content="content")]
pub enum RoleType {
    assistant,
    user,
    system,
}

#[derive(Deserialize, Debug, Clone)]
struct ResponseErrorContent {
    message:String,
}

#[derive(Deserialize, Debug)]
struct OpenAIErrorResponse {
    error: ResponseErrorContent,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FuncResponse {
    pub name: String,
    pub arguments: HashMap<String, String>,
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

#[derive(Deserialize, Debug)]
struct OpenAIResponse {
    choices: Vec<ResponseChoiseUnit>,
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
    functions: Vec<FuncUnit>,
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
    pub parameters: FuncParams,
}

pub type OError = Box<dyn std::error::Error + Send + Sync>;

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

pub type Result<T> 
    = std::result::Result<T, OError>;

pub async fn ask(
        messages: Vec<RequestMessageUnit>, functions: Vec<FuncUnit>
    ) -> Result<ResponseMessageUnit> {

    //let mut re:Vec<String> = Vec::new();
    //for i in messages.clone() {
    //    match i {
    //        RoleType::user(s) => {
    //            re.push(s)
    //        },
    //        _ => {
    //        }
    //    }
    //};
    //return Ok(re.join(""));


    let https = HttpsConnector::new();
    let client = Client::builder().build(https);
    let uri = "https://api.openai.com/v1/chat/completions";

    let model = String::from("gpt-3.5-turbo");
    //let stop = String::from("\n");

    let mut api_key = String::new();

    match env::var("OPENAI_API_KEY") {
        Ok(x) => {
            api_key = x;
        },
        Err(_) => {
            eprintln!("Need OPENAI_API_KEY");
            return Err(Box::new(OpenAIError::from_str("Need OPENAI_API_KEY")))
        }
    };
    let auth_header_val = format!("Bearer {}", api_key);

    let openai_request = OpenAIRequest {
        model,
        messages,
        functions,
    };

    let body = Body::from(serde_json::to_string(&openai_request)?);

    let req = Request::post(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("Authorization", &auth_header_val)
        .body(body)?;

    let res = client.request(req).await?;
    match res.status() {
        StatusCode::OK => {
            let mut body = hyper::body::aggregate(res).await?;
            //println!("openai res body: {:?}", String::from_utf8(body.reader()));
            let b = body.copy_to_bytes(1024000);
            let json: OpenAIResponse = match serde_json
                ::from_reader(body.reader()) {
                    Ok(j) => j,
                    Err(e) => {
                        eprintln!("json parse Error: {:?}", e);
                        let j: Value = match serde_json
                            ::from_reader(b.reader()){
                            Ok(jj) => {
                                eprintln!("json parse Error1: {:?}", jj);
                                jj
                            },
                            Err(ee) => {
                                eprintln!("json parse Error2 : {:?}", e);
                                return Err(Box::new(e))
                            }
                        };
                        return Err(Box::new(e))
                    }
            };
            println!("openai res json, {:?}", json);
            return Ok(json.choices[0].clone().message);
            //match clone() {
            //    ResponseMessageUnit{message:RoleType::assistant(x)} => {
            //        Ok(x)
            //    },
            //    ResponseMessageUnit{message:RoleType::user(x)} => {
            //        Ok(format!("Human: {}", x))
            //    },
            //    ResponseMessageUnit{message:RoleType::system(x)} => {
            //        Ok(format!("System: {}", x))
            //    },
            //    ResponseMessageUnit{message:RoleType::None} => {
            //        Ok("".to_string())
            //    }
            //}
        },
        StatusCode::BAD_REQUEST => {
            let body = hyper::body::aggregate(res).await?;
            let error: OpenAIErrorResponse = serde_json::from_reader(body.reader())?;
            Err(Box::new(OpenAIError::from_string(error.error.message)))
            //Ok(error.error.message)
        },
        _ => {
            eprintln!("Error res: {:?}", res);
            let body = hyper::body::aggregate(res).await?;
            let error: OpenAIErrorResponse = serde_json::from_reader(body.reader())?;
            Err(Box::new(OpenAIError::from_string(error.error.message)))
        }
    }
}

pub async fn draw(prompt: String, n: i32, size: String) -> Result<String> {

    let https = HttpsConnector::new();
    let client = Client::builder().build(https);
    let uri = "https://api.openai.com/v1/images/generations";

    let mut api_key = String::new();

    match env::var("OPENAI_API_KEY") {
        Ok(x) => {
            api_key = x;
        },
        Err(e) => {
            println!("Need OPENAI_API_KEY");
            return Ok("".to_string());
        }
    };
    let auth_header_val = format!("Bearer {}", api_key);

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

    let body = Body::from(serde_json::to_string(&openai_request)?);

    let req = Request::post(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("Authorization", &auth_header_val)
        .body(body)?;

    let res = client.request(req).await?;
    match res.status() {
        StatusCode::OK => {
            let body = hyper::body::aggregate(res).await?;
            let json: OpenAIImageResponse
                = serde_json::from_reader(body.reader())?;
            let mut result: Vec<String> = Vec::new();
            for i in json.data {
                result.push(i.url)
            }
            return Ok(result.join("\n"))
        },
        StatusCode::BAD_REQUEST => {
            let body = hyper::body::aggregate(res).await?;
            let error: OpenAIErrorResponse = serde_json::from_reader(body.reader())?;
            Err(Box::new(OpenAIError::from_string(error.error.message)))
        },
        _ => {
            eprintln!("Error res: {:?}", res);
            let body = hyper::body::aggregate(res).await?;
            let error: OpenAIErrorResponse = serde_json::from_reader(body.reader())?;
            Err(Box::new(OpenAIError::from_string(error.error.message)))
        }
    }
}
