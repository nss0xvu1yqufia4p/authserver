use actix_web::{App, HttpServer};
use authserver::{configure, AuthService};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let svc = AuthService::from_env().unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(1);
    });

    HttpServer::new(move || {
        App::new().configure(|c| configure(c, svc.clone()))
    })
    .bind("0.0.0.0:80")?
    .run()
    .await
}

