use actix_web::{get, web, App, HttpServer, HttpRequest, HttpMessage, http, HttpResponse, Result};
use std::env;
use rand::Rng;
use base64::{encode, decode, DecodeError};
use sha2::{Sha256, Digest};

fn hash(input: Vec<u8>) -> Result<Vec<u8>, DecodeError> {
    let mut input = input.clone();
    let secret_base64 = env::var("AUTH_SERVER_SECRET").unwrap();
    let mut secret_bytes = decode(&secret_base64)?;
    input.append(&mut secret_bytes);
    let mut hasher = Sha256::new();
    hasher.update(&input);
    let hash = hasher.finalize();
    let truncated_hash = &hash[0..16];
    let truncated_hash_byte_vec: Vec<u8> = truncated_hash.iter().cloned().collect();
    return Ok(truncated_hash_byte_vec);
}

fn validate_cookie(cookie_base64: String) -> Result<bool, DecodeError> {
    let cookie = decode(&cookie_base64)?;
    let cookie_token = &cookie[0..8];
    let cookie_hash: Vec<u8> = (&cookie[8..24]).to_vec();
    let token_hash = hash((&cookie_token).to_vec())?;
    Ok(token_hash == cookie_hash)
}

fn generate_cookie() -> Result<String, DecodeError> {
    let token = rand::thread_rng().gen::<[u8; 8]>();
    let hash = hash((&token).to_vec())?;
    let cookie_bytes: Vec<u8> = token.iter().cloned().chain(hash.iter().cloned()).collect();
    let cookie_base64 =  encode(&cookie_bytes);
    return Ok(cookie_base64);
}

#[get("/{token}")]
async fn auth(web::Path(token): web::Path<String>) -> Result<HttpResponse> {
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
    if let Some(c) = req.cookie("auth_token") {
       if let Ok(_) = validate_cookie(c.value().to_string()) {
           return Ok(HttpResponse::Ok().finish());
       }
    }
    return Ok(HttpResponse::Forbidden().finish());
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Auth Server Starting");
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
