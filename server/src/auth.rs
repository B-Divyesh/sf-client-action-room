use std::{
    collections::HashMap,
    env,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

use axum::http::{header, HeaderMap, StatusCode};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::Deserialize;

use crate::demo::ApiError;

const DEFAULT_TENANT: &str = "35c6fe40-0ec0-46b6-98c6-213ad4de6650";
const DEFAULT_SUBDOMAIN: &str = "sociobotcustomers";
const DEFAULT_CLIENT: &str = "25c704f4-465a-47af-80ab-2c489466b697";

#[derive(Clone)]
pub struct AuthService {
    tenant_id: String,
    client_id: String,
    discovery_url: String,
    client: reqwest::Client,
    cache: Arc<RwLock<Option<KeyCache>>>,
    accept_test_tokens: bool,
}

#[derive(Clone)]
struct KeyCache {
    issuer: String,
    keys: HashMap<String, (String, String)>,
    fetched_at: Instant,
}

#[derive(Deserialize)]
struct Discovery {
    issuer: String,
    jwks_uri: String,
}

#[derive(Deserialize)]
struct JwkSet {
    keys: Vec<Jwk>,
}

#[derive(Deserialize)]
struct Jwk {
    kid: String,
    n: String,
    e: String,
    kty: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct StaffClaims {
    pub oid: String,
    pub tid: String,
    #[serde(default)]
    pub name: String,
    #[serde(default, alias = "preferred_username")]
    pub email: String,
}

impl Default for AuthService {
    fn default() -> Self {
        Self::from_env()
    }
}

impl AuthService {
    pub fn from_env() -> Self {
        let tenant_id = env::var("ENTRA_TENANT_ID").unwrap_or_else(|_| DEFAULT_TENANT.into());
        let subdomain =
            env::var("ENTRA_TENANT_SUBDOMAIN").unwrap_or_else(|_| DEFAULT_SUBDOMAIN.into());
        let client_id = env::var("ENTRA_CLIENT_ID").unwrap_or_else(|_| DEFAULT_CLIENT.into());
        let discovery_url = format!(
            "https://{subdomain}.ciamlogin.com/{tenant_id}/v2.0/.well-known/openid-configuration"
        );
        Self {
            tenant_id,
            client_id,
            discovery_url,
            client: reqwest::Client::new(),
            cache: Arc::new(RwLock::new(None)),
            accept_test_tokens: env::var("AUTH_TEST_MODE").as_deref() == Ok("1"),
        }
    }

    pub fn for_tests() -> Self {
        let mut service = Self::from_env();
        service.accept_test_tokens = true;
        service
    }

    pub async fn verify(&self, headers: &HeaderMap) -> Result<StaffClaims, ApiError> {
        let token = headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .ok_or_else(|| {
                ApiError::unauthorized("Sign in with your Sociobot account to open this workspace.")
            })?;
        if self.accept_test_tokens {
            if let Some(oid) = token
                .strip_prefix("test:")
                .filter(|value| !value.is_empty())
            {
                return Ok(StaffClaims {
                    oid: oid.to_owned(),
                    tid: self.tenant_id.clone(),
                    name: format!("Owner {oid}"),
                    email: format!("{oid}@example.test"),
                });
            }
        }
        let header = decode_header(token).map_err(|_| invalid_token())?;
        if header.alg != Algorithm::RS256 {
            return Err(invalid_token());
        }
        let kid = header.kid.ok_or_else(invalid_token)?;
        let cache = self.keys().await?;
        let (n, e) = cache.keys.get(&kid).ok_or_else(invalid_token)?;
        let key = DecodingKey::from_rsa_components(n, e).map_err(|_| invalid_token())?;
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[self.client_id.as_str()]);
        validation.set_issuer(&[cache.issuer.as_str()]);
        validation.validate_nbf = true;
        let claims = decode::<StaffClaims>(token, &key, &validation)
            .map_err(|_| invalid_token())?
            .claims;
        if claims.tid != self.tenant_id || claims.oid.is_empty() {
            return Err(invalid_token());
        }
        Ok(claims)
    }

    async fn keys(&self) -> Result<KeyCache, ApiError> {
        if let Some(cache) = self.cache.read().expect("auth cache poisoned").clone() {
            if cache.fetched_at.elapsed() < Duration::from_secs(3_600) {
                return Ok(cache);
            }
        }
        let discovery: Discovery = self
            .client
            .get(&self.discovery_url)
            .send()
            .await
            .map_err(|_| auth_unavailable())?
            .error_for_status()
            .map_err(|_| auth_unavailable())?
            .json()
            .await
            .map_err(|_| auth_unavailable())?;
        let jwks: JwkSet = self
            .client
            .get(&discovery.jwks_uri)
            .send()
            .await
            .map_err(|_| auth_unavailable())?
            .error_for_status()
            .map_err(|_| auth_unavailable())?
            .json()
            .await
            .map_err(|_| auth_unavailable())?;
        let keys = jwks
            .keys
            .into_iter()
            .filter(|key| key.kty == "RSA")
            .map(|key| (key.kid, (key.n, key.e)))
            .collect();
        let cache = KeyCache {
            issuer: discovery.issuer,
            keys,
            fetched_at: Instant::now(),
        };
        *self.cache.write().expect("auth cache poisoned") = Some(cache.clone());
        Ok(cache)
    }
}

fn invalid_token() -> ApiError {
    ApiError::new(
        StatusCode::UNAUTHORIZED,
        "invalid_token",
        "Your sign-in has expired. Sign in again.",
    )
}
fn auth_unavailable() -> ApiError {
    ApiError::new(
        StatusCode::SERVICE_UNAVAILABLE,
        "identity_unavailable",
        "Sign-in verification is unavailable. Try again in a moment.",
    )
}
