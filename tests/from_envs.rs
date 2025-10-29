// tests/from_envs.rs

use authserver::AuthService;
use base64::{engine::general_purpose, Engine as _};
use serial_test::serial;
use std::env;

/// Utility: set var, returning previous value to restore later.
fn set_env(name: &str, val: &str) -> Option<String> {
    let old = env::var(name).ok();
    env::set_var(name, val);
    old
}

/// Utility: restore a var to previous value (or remove if none).
fn restore_env(name: &str, old: Option<String>) {
    match old {
        Some(v) => env::set_var(name, v),
        None => env::remove_var(name),
    }
}

/// Guard that restores a single env var on drop.
struct EnvGuard {
    name: &'static str,
    old: Option<String>,
}
impl EnvGuard {
    fn new(name: &'static str, val: &str) -> Self {
        let old = set_env(name, val);
        Self { name, old }
    }
}
impl Drop for EnvGuard {
    fn drop(&mut self) {
        restore_env(self.name, self.old.take());
    }
}

#[test]
#[serial] // serialize to avoid cross-test env races
fn from_env_success() {
    // Prepare base64(secret) — from_env() uses STANDARD decode
    let secret_b64 = general_purpose::STANDARD.encode(b"supersecret");

    let _g1 = EnvGuard::new("AUTH_SERVER_SECRET", &secret_b64);
    let _g2 = EnvGuard::new("DOMAIN_NAME", "example.com");

    let svc = AuthService::from_env().expect("from_env should succeed");

    // Smoke-check: service works with decoded secret & domain
    let cookie = svc.generate_cookie();
    assert!(svc.validate_cookie(&cookie).unwrap());
    assert_eq!(svc.domain(), "example.com");
}

#[test]
#[serial]
fn from_env_missing_secret() {
    // Ensure AUTH_SERVER_SECRET absent; DOMAIN_NAME present
    let _g1 = EnvGuard::new("DOMAIN_NAME", "example.org");
    env::remove_var("AUTH_SERVER_SECRET");

    // Do not use unwrap_err() to avoid requiring AuthService: Debug
    let err = AuthService::from_env()
        .err()
        .expect("expected an error when AUTH_SERVER_SECRET is missing");
    assert!(
        !err.is_empty(),
        "expected a non-empty error message when AUTH_SERVER_SECRET is missing"
    );
}

#[test]
#[serial]
fn from_env_bad_secret_base64() {
    let _g1 = EnvGuard::new("AUTH_SERVER_SECRET", "!!!not-base64!!!");
    let _g2 = EnvGuard::new("DOMAIN_NAME", "example.net");

    let err = AuthService::from_env()
        .err()
        .expect("expected base64 decode error for AUTH_SERVER_SECRET");
    // from_env formats this as: "AUTH_SERVER_SECRET base64 decode failed: {e}"
    assert!(
        err.contains("base64"),
        "expected base64 decode failure, got: {err}"
    );
}

#[test]
#[serial]
fn from_env_missing_domain() {
    let secret_b64 = general_purpose::STANDARD.encode(b"supersecret");
    let _g1 = EnvGuard::new("AUTH_SERVER_SECRET", &secret_b64);
    env::remove_var("DOMAIN_NAME");

    let err = AuthService::from_env()
        .err()
        .expect("expected an error when DOMAIN_NAME is missing");
    assert!(
        !err.is_empty(),
        "expected a non-empty error message when DOMAIN_NAME is missing"
    );
}

