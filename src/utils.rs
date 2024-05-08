use std::io::Write;
use std::{
    env,
    error,
    fmt,
    fmt::Display,
    fs::File,
};
use std::collections::HashMap;

//use bytes::Bytes;

//use http::status::StatusCode;

use tokio;
use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt as _, self};

use http_body::Body as HttpBody;
use http_body_util::{
    Full,
    BodyExt,
};
use hyper_util::rt::TokioIo;

use hyper::{self, Method};
use hyper::{
    body::{Buf, Bytes, Body, Incoming},
    header,
    Request, 
    Response,
    http::request::Builder,
    http::status::StatusCode,
};
use hyper_tls::HttpsConnector;

use crate::errors::{
    OpenAIError,
    OpenAIResult,
    OpenAIErrorResponse,
};

pub async fn request(
        url: String,
        req_data: String,
    ) ->  OpenAIResult<Response<Incoming>> {
    
    let url = url.parse::<hyper::Uri>().unwrap();
    
    let host = url.host().expect("uri has no host");
    let port = url.port_u16().unwrap_or(80);
    let addr = format!("{}:{}", host, port);
    let stream = TcpStream::connect(addr).await?;
    let io = TokioIo::new(stream);

    let (mut sender, conn) = hyper::client::conn::http1::handshake(io)
                             .await.unwrap();
    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            println!("Connection failed: {:?}", err);
        }
    });

    let authority = url.authority().unwrap().clone();
    let path = url.path();

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

    let body = Full::new(Bytes::from(req_data));
    //let body = Body::from(req_data);

    let req = Request::builder()
        .method(Method::POST)
        .uri(path)
        .header(hyper::header::HOST, authority.as_str())
        .header(header::CONTENT_TYPE, "application/json")
        .header("Authorization", &auth_header_val)
        .body(body)
        .unwrap()
        ;

    let res = sender.send_request(req).await?;
    return Ok(res);

    //match res.status() {
    //    StatusCode::OK => {
    //        //let body = hyper::body::aggregate(res).await?;
    //        //println!("openai res body: {:?}", String::from_utf8(body.reader()));
    //        while let Some(next) = res.frame().await {
    //            let frame = next?;
    //            if let Some(chunk) = frame.data_ref(){
    //                output
    //                    .write_all(chunk)
    //                    .map_err(|e|{
    //                        eprintln!("download speed ERROR: {:?}", e)
    //                    });
    //            }
    //        }
    //        return Ok(OpenAISpeedResult{});
    //    },
    //    StatusCode::BAD_REQUEST => {
    //        let body = hyper::body::aggregate(res).await?;
    //        let error: OpenAIErrorResponse = serde_json::from_reader(body.reader())?;
    //        Err(Box::new(OpenAIError::from_string(error.error.message)))
    //    },
    //    _ => {
    //        eprintln!("Error res: {:?}", res);
    //        let body = hyper::body::aggregate(res).await?;
    //        let error: OpenAIErrorResponse = serde_json::from_reader(body.reader())?;
    //        Err(Box::new(OpenAIError::from_string(error.error.message)))
    //    }
    //}
}
