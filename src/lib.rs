// src/lib.rs

use actix_web::{get, web, HttpRequest, HttpResponse, http, cookie::Cookie};
use base64::{engine::general_purpose, Engine as _};
use rand::{RngCore, rngs::StdRng, SeedableRng};
use sha2::{Sha256, Digest};
use std::env;
use std::sync::Arc;

const TOKEN_LEN: usize = 8;
const HASH_LEN: usize = 16;

#[derive(Clone)]
pub struct AuthService {
    secret: Arc<Vec<u8>>,
    domain: String,
    rng_seed: Option<u64>,
}

impl AuthService {
    pub fn new(secret_bytes: Vec<u8>, domain: String) -> Self {
        Self { secret: Arc::new(secret_bytes), domain, rng_seed: None }
    }

    pub fn from_env() -> Result<Self, String> {
        let secret_b64 = env::var("AUTH_SERVER_SECRET").map_err(|e| e.to_string())?;
        let secret = general_purpose::STANDARD
            .decode(secret_b64.as_bytes())
            .map_err(|e| format!("AUTH_SERVER_SECRET base64 decode failed: {e}"))?;
        let domain = env::var("DOMAIN_NAME").map_err(|e| e.to_string())?;
        Ok(Self::new(secret, domain))
    }

    /// Make RNG deterministic (needed by integration tests).
    /// Kept public so integration tests (compiled as separate crates) can use it.
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.rng_seed = Some(seed);
        self
    }

    pub fn hash(&self, input: &[u8]) -> [u8; HASH_LEN] {
        let mut hasher = Sha256::new();
        hasher.update(input);
        hasher.update(&*self.secret);
        let digest = hasher.finalize();
        let mut out = [0u8; HASH_LEN];
        out.copy_from_slice(&digest[..HASH_LEN]);
        out
    }

    pub fn generate_cookie(&self) -> String {
        let mut token = [0u8; TOKEN_LEN];
        match self.rng_seed {
            Some(seed) => {
                let mut rng = StdRng::seed_from_u64(seed);
                rng.fill_bytes(&mut token);
            }
            None => rand::thread_rng().fill_bytes(&mut token),
        }
        let hash = self.hash(&token);
        let mut buf = [0u8; TOKEN_LEN + HASH_LEN];
        buf[..TOKEN_LEN].copy_from_slice(&token);
        buf[TOKEN_LEN..].copy_from_slice(&hash);
        general_purpose::URL_SAFE_NO_PAD.encode(buf)
    }

    pub fn validate_cookie(&self, cookie_b64: &str) -> Result<bool, String> {
        let data = general_purpose::URL_SAFE_NO_PAD
            .decode(cookie_b64.as_bytes())
            .map_err(|e| format!("cookie base64 decode failed: {e}"))?;
        if data.len() != TOKEN_LEN + HASH_LEN {
            return Ok(false);
        }
        let token = &data[..TOKEN_LEN];
        let expected = self.hash(token);
        let provided = &data[TOKEN_LEN..];
        Ok(constant_time_eq(&expected, provided))
    }

    pub fn domain(&self) -> &str { &self.domain }
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() { return false; }
    let mut diff = 0u8;
    for i in 0..a.len() { diff |= a[i] ^ b[i]; }
    diff == 0
}

#[get("/{token}")]
pub async fn auth(path: web::Path<String>, svc: web::Data<AuthService>) -> HttpResponse {
    let token_b64 = path.into_inner();
    match svc.validate_cookie(&token_b64) {
        Ok(true) => {
            HttpResponse::Found()
                .cookie(
                    Cookie::build("auth_token", token_b64)
                        .domain(svc.domain().to_string())
                        .path("/")
                        .secure(true)
                        .http_only(true)
                        .finish()
                )
                .insert_header((http::header::LOCATION, "/"))
                .finish()
        }
        _ => HttpResponse::Forbidden().finish(),
    }
}

#[get("/")]
pub async fn index(req: HttpRequest, svc: web::Data<AuthService>) -> HttpResponse {
    if let Some(c) = req.cookie("auth_token") {
        if let Ok(true) = svc.validate_cookie(c.value()) {
            return HttpResponse::Ok().finish();
        }
    }
    HttpResponse::Forbidden().finish()
}

pub fn configure(cfg: &mut web::ServiceConfig, svc: AuthService) {
    cfg.app_data(web::Data::new(svc))
        .service(index)
        .service(auth);
}

