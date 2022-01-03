use actix_web::{get, web, App, HttpServer, http,  HttpResponse, Result};
use std::env;

#[get("/{token}")]
async fn auth(web::Path(token): web::Path<String>) -> Result<HttpResponse> {
    let static_auth_token = env::var("AUTH_TOKEN").unwrap();
    println!("Token {}!", token);
    println!("Static Token {}!", static_auth_token);
    return Ok(
        HttpResponse::Ok()
        .header("X-TEST", "value")
        .header(http::header::CONTENT_TYPE, "application/json")
        .finish()
    );
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Auth server starting");
    if let Err(e) = env::var("AUTH_TOKEN") {
        eprintln!("{}", e);
        std::process::exit(1);
    }
    HttpServer::new(|| App::new()
                    .service(auth)
    )
    .bind("0.0.0.0:80")?
    .run()
    .await
}
