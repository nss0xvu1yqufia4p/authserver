use actix_web::{get, web, App, HttpServer, HttpRequest, HttpMessage, http, HttpResponse, Result};
use serde_json::{json, Value, Map};
use std::error;
use std::env;
use rand::Rng;
use base64::{encode, decode, DecodeError};
use sha2::{Sha256, Sha512, Digest};

fn hash(input: Vec<u8>) -> Vec<u8> {
    let mut input = input.clone();
    let secret_base64 = env::var("AUTH_SERVER_SECRET").unwrap();

    println!("{:?}", &input);
    println!("{:?}", &secret_base64);
    let mut secret_bytes = decode(&secret_base64).unwrap();
    println!("{:?}", &secret_bytes);

    input.append(&mut secret_bytes);
    let mut hasher = Sha256::new();
    hasher.update(&input);
    let hash = hasher.finalize();
    let truncated_hash = &hash[0..16];
    let truncated_hash_byte_vec: Vec<u8> = truncated_hash.iter().cloned().collect();
    return truncated_hash_byte_vec;
}

fn validate_cookie(cookie_base64: String) -> Result<bool, DecodeError> {
    let cookie = decode(&cookie_base64)?;
    println!("{:?}", &cookie);
    let cookie_token = &cookie[0..8];
    println!("{:?}", &cookie_token);
    let cookie_hash: Vec<u8> = (&cookie[8..24]).to_vec();
    println!("{:?}", &cookie_hash);
    let token_hash = hash((&cookie_token).to_vec());
    println!("{:?}", &token_hash);
    Ok(token_hash == cookie_hash)
}

fn generate_cookie() -> String {
    let token = rand::thread_rng().gen::<[u8; 8]>();
    let hash = hash((&token).to_vec());
    let cookie_bytes: Vec<u8> = token.iter().cloned().chain(hash.iter().cloned()).collect();
    let cookie_base64 =  base64::encode(&cookie_bytes);
    return cookie_base64;
}

#[get("/{token}")]
async fn auth(web::Path(token): web::Path<String>) -> Result<HttpResponse> {
    println!("auth: {}", &token);
    let domain = env::var("DOMAIN_NAME").unwrap();
    match validate_cookie(token.clone()) {
        Ok(true) => Ok(
            HttpResponse::Found()
            .cookie(
                http::Cookie::build("auth_token", &token)
                .domain(domain)
                .path("/")
                .secure(true)
                .http_only(true)
                .finish()
            )
            .header(http::header::LOCATION, "/")
            .finish()
        ),
        _ => Ok(
            HttpResponse::Forbidden().finish()
        ) 
    }
}

#[get("/")]
async fn index(req: HttpRequest) -> Result<HttpResponse> {
    println!("index");
    if let Some(c) = req.cookie("auth_token") {
       if let Ok(_) = validate_cookie(c.value().to_string()) {
           println!("qqqindex");
           return Ok(HttpResponse::Ok().finish());
       }
    }
    println!("wwwindex");
    return Ok(HttpResponse::Forbidden().finish());
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Auth server starting");
    if let Err(e) = env::var("AUTH_SERVER_SECRET") {
        eprintln!("{}", e);
        std::process::exit(1);
    }
    if let Err(e) = env::var("DOMAIN_NAME") {
        eprintln!("{}", e);
        std::process::exit(1);
    }
    HttpServer::new(|| App::new()
                    .service(index)
                    .service(auth)
    )
    .bind("0.0.0.0:80")?
    .run()
    .await
}
