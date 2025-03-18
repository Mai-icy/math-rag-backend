use chrono::format::StrftimeItems;
use reqwest::{header::HeaderValue, Error};
use reqwest::header::HeaderMap;
use reqwest::Client;
use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use sha2::{Sha256, Digest};
use hmac::{Hmac, Mac};
use base64::{engine::general_purpose, Engine as _};
use dotenv::dotenv;
use std::fs;
use std::env;

fn get_rfc1123_time() -> String {
    let now: DateTime<Utc> = Utc::now();
    let format = StrftimeItems::new("%a, %d %b %Y %H:%M:%S GMT");
    now.format_with_items(format).to_string()
}

fn assmble_header(body: Value) -> HeaderMap{
    let mut hasher = Sha256::new();
    let body_str = body.to_string();
    hasher.update(body_str.as_bytes());
    let hash_result = hasher.finalize();
    let sha256_body = general_purpose::STANDARD.encode(&hash_result);
    let mut digest = String::from("SHA-256=");
    digest.push_str(&sha256_body);

    let mut ori_signatrue = String::new();
    ori_signatrue.push_str("host: rest-api.xfyun.cn\n");
    ori_signatrue.push_str(&format!("date: {}\n", get_rfc1123_time()));
    ori_signatrue.push_str("POST /v2/itr HTTP/1.1\n");
    ori_signatrue.push_str(&format!("digest: {}", digest));

    let secret = env::var("API_SECRET").expect("no api secret");
    let key = env::var("API_KEY").expect("no api_key");
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).expect("HMAC can take a key of any size");
    mac.update(ori_signatrue.as_bytes());
    let hmac_result = mac.finalize();
    let new_signatrue = general_purpose::STANDARD.encode(hmac_result.into_bytes());

    let auth = format!("api_key=\"{}\", algorithm=\"hmac-sha256\", headers=\"host date request-line digest\", signature=\"{}\"", 
                        key, 
                        new_signatrue);

    let mut headers = HeaderMap::new();
    headers.insert("Host", HeaderValue::from_static("rest-api.xfyun.cn"));
    headers.insert("Date", HeaderValue::from_str(&get_rfc1123_time()).unwrap());
    headers.insert("Digest", HeaderValue::from_str(&digest).unwrap());
    headers.insert("Authorization", HeaderValue::from_str(&auth).unwrap());
    headers
}

pub async fn formula_discern(img_base64: &String) -> Result<Value, Error> {
    let url = "https://rest-api.xfyun.cn/v2/itr";

    let data = json!({
        "common": {
            "app_id": env::var("APP_ID").unwrap()
        },
        "business": {
            "ent": "teach-photo-print",
            "aue": "raw"
        },
        "data": {
            "image": img_base64
        }
    });

    let header = assmble_header(json!({}));
    let client = Client::new();
    let response = client
        .post(url)
        .headers(header)
        .json(&data)
        .send()
        .await?;

    let json_response: Value = response.json().await?;
    Ok(json_response)
}


pub async fn img2latex(img_base64: &String) -> Result<String, Error> {
    let res_json = formula_discern(img_base64).await?;
    
    let contexts = res_json["data"]["region"].as_array();
    
    let mut text = String::new();

    if let Some(contexts) = contexts {
        for region in contexts {
            if let Some(line) = region["recog"]["content"].as_str() {
                // 移除 ifly-latex-begin 和 ifly-latex-end
                let cleaned_line = line
                    .replace(" ifly-latex-begin ", "$")
                    .replace(" ifly-latex-end ", "$");
                
                use std::fmt::Write;
                write!(text, "{}", cleaned_line).ok();
            }
        }
    }
    
    Ok(text)
}
