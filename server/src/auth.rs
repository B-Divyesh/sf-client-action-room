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
    keys: HashMap<String, DecodingKey>,
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
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .expect("identity HTTP client must build"),
            cache: Arc::new(RwLock::new(None)),
            accept_test_tokens: env::var("AUTH_TEST_MODE").as_deref() == Ok("1"),
        }
    }

    pub fn for_tests() -> Self {
        let mut service = Self::from_env();
        service.accept_test_tokens = true;
        service
    }

    pub async fn warm(&self) -> Result<(), ApiError> {
        self.keys(false).await.map(|_| ())
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
        let mut cache = self.keys(false).await?;
        if !cache.keys.contains_key(&kid) {
            cache = self.keys(true).await?;
        }
        let key = cache.keys.get(&kid).ok_or_else(invalid_token)?;
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[self.client_id.as_str()]);
        validation.set_issuer(&[cache.issuer.as_str()]);
        validation.validate_nbf = true;
        let claims = decode::<StaffClaims>(token, key, &validation)
            .map_err(|_| invalid_token())?
            .claims;
        if claims.tid != self.tenant_id || claims.oid.is_empty() {
            return Err(invalid_token());
        }
        Ok(claims)
    }

    async fn keys(&self, force_refresh: bool) -> Result<KeyCache, ApiError> {
        if !force_refresh {
            if let Some(cache) = self.cache.read().expect("auth cache poisoned").clone() {
                if cache.fetched_at.elapsed() < Duration::from_secs(3_600) {
                    return Ok(cache);
                }
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
            .filter_map(|key| {
                DecodingKey::from_rsa_components(&key.n, &key.e)
                    .ok()
                    .map(|decoding| (key.kid, decoding))
            })
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

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{encode, EncodingKey, Header};
    use rsa::{
        pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding},
        rand_core::OsRng,
        RsaPrivateKey, RsaPublicKey,
    };
    use serde::Serialize;
    use std::sync::OnceLock;

    const ISSUER: &str = "https://issuer.example.test/tenant/v2.0";

    fn fixture_keys() -> &'static (Vec<u8>, Vec<u8>) {
        static KEYS: OnceLock<(Vec<u8>, Vec<u8>)> = OnceLock::new();
        KEYS.get_or_init(|| {
            let private = RsaPrivateKey::new(&mut OsRng, 2_048).unwrap();
            let public = RsaPublicKey::from(&private);
            (
                private
                    .to_pkcs8_pem(LineEnding::LF)
                    .unwrap()
                    .as_bytes()
                    .to_vec(),
                public
                    .to_public_key_pem(LineEnding::LF)
                    .unwrap()
                    .as_bytes()
                    .to_vec(),
            )
        })
    }

    #[derive(Serialize)]
    struct FixtureClaims<'a> {
        oid: &'a str,
        tid: &'a str,
        name: &'a str,
        email: &'a str,
        aud: &'a str,
        iss: &'a str,
        exp: usize,
        nbf: usize,
    }

    fn fixture_service(kids: &[&str]) -> AuthService {
        let keys = kids
            .iter()
            .map(|kid| {
                (
                    (*kid).to_owned(),
                    DecodingKey::from_rsa_pem(&fixture_keys().1).unwrap(),
                )
            })
            .collect();
        AuthService {
            tenant_id: DEFAULT_TENANT.into(),
            client_id: DEFAULT_CLIENT.into(),
            discovery_url: "http://127.0.0.1:1/unavailable".into(),
            client: reqwest::Client::new(),
            cache: Arc::new(RwLock::new(Some(KeyCache {
                issuer: ISSUER.into(),
                keys,
                fetched_at: Instant::now(),
            }))),
            accept_test_tokens: false,
        }
    }

    fn token(kid: &str, overrides: impl FnOnce(&mut FixtureClaims<'_>)) -> String {
        let now = chrono::Utc::now().timestamp() as usize;
        let mut claims = FixtureClaims {
            oid: "staff-oid",
            tid: DEFAULT_TENANT,
            name: "Morgan Owner",
            email: "morgan@example.test",
            aud: DEFAULT_CLIENT,
            iss: ISSUER,
            exp: now + 300,
            nbf: now.saturating_sub(10),
        };
        overrides(&mut claims);
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(kid.into());
        encode(
            &header,
            &claims,
            &EncodingKey::from_rsa_pem(&fixture_keys().0).unwrap(),
        )
        .unwrap()
    }

    fn bearer(token: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            format!("Bearer {token}").parse().unwrap(),
        );
        headers
    }

    #[tokio::test]
    async fn validates_signature_audience_tenant_issuer_and_time() {
        let service = fixture_service(&["current"]);
        let valid = token("current", |_| {});
        assert_eq!(
            service.verify(&bearer(&valid)).await.unwrap().oid,
            "staff-oid"
        );

        let wrong_audience = token("current", |claims| claims.aud = "another-client");
        assert!(service.verify(&bearer(&wrong_audience)).await.is_err());
        let wrong_tenant = token("current", |claims| claims.tid = "another-tenant");
        assert!(service.verify(&bearer(&wrong_tenant)).await.is_err());
        let wrong_issuer = token("current", |claims| {
            claims.iss = "https://wrong.example.test"
        });
        assert!(service.verify(&bearer(&wrong_issuer)).await.is_err());
        let expired = token("current", |claims| claims.exp = 1);
        assert!(service.verify(&bearer(&expired)).await.is_err());
        let future = token("current", |claims| {
            claims.nbf = (chrono::Utc::now().timestamp() + 300) as usize
        });
        assert!(service.verify(&bearer(&future)).await.is_err());
    }

    #[tokio::test]
    async fn selects_a_rotated_key_by_kid_and_rejects_unknown_keys() {
        let service = fixture_service(&["old", "new"]);
        let rotated = token("new", |_| {});
        assert!(service.verify(&bearer(&rotated)).await.is_ok());
        let unknown = token("unknown", |_| {});
        assert!(service.verify(&bearer(&unknown)).await.is_err());
    }
}
