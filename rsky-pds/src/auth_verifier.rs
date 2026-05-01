use crate::account_manager::helpers::account::{ActorAccount, AvailabilityFlags};
use crate::account_manager::helpers::auth::CustomClaimObj;
use crate::account_manager::AccountManager;
use crate::apis::ApiError;
use crate::xrpc_server::auth::{verify_jwt as verify_service_jwt_server, ServiceJwtPayload};
use crate::{SharedCompositeResolver, SharedIdResolver};
use anyhow::{bail, Result};
use base64::{
    engine::general_purpose::{STANDARD as base64pad, URL_SAFE as base64url},
    Engine as _,
};
use jwt_simple::claims::Audiences;
use jwt_simple::prelude::*;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::State;
use rsky_common::env::env_str;
use rsky_common::get_verification_material;
use rsky_identity::did::atproto_data::get_did_key_from_multibase;
use rsky_identity::did::capability::{DidCapability, DidError};
use rsky_identity::did::composite_resolver::CompositeDidResolver;
use rsky_identity::types::DidDocument;
use secp256k1::{Keypair, Secp256k1, SecretKey};
use std::env;
use std::str;
use std::sync::LazyLock;
use thiserror::Error;

const INFINITY: u64 = u64::MAX;

pub static PDS_JWT_KEYPAIR: LazyLock<ES256kKeyPair> = LazyLock::new(|| {
    let secp = Secp256k1::new();
    let private_key = env::var("PDS_JWT_KEY_K256_PRIVATE_KEY_HEX").unwrap();
    let secret_key = SecretKey::from_slice(&hex::decode(private_key.as_bytes()).unwrap()).unwrap();
    let jwt_key = Keypair::from_secret_key(&secp, &secret_key);
    ES256kKeyPair::from_bytes(jwt_key.secret_bytes().as_slice()).unwrap()
});

#[derive(PartialEq, Clone, Debug)]
pub enum AuthScope {
    Access,
    Refresh,
    AppPass,
    AppPassPrivileged,
    SignupQueued,
}

impl AuthScope {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuthScope::Access => "com.atproto.access",
            AuthScope::Refresh => "com.atproto.refresh",
            AuthScope::AppPass => "com.atproto.appPass",
            AuthScope::AppPassPrivileged => "com.atproto.appPassPrivileged",
            AuthScope::SignupQueued => "com.atproto.signupQueued",
        }
    }

    pub fn from_str(scope: &str) -> Result<Self> {
        match scope {
            "com.atproto.access" => Ok(AuthScope::Access),
            "com.atproto.refresh" => Ok(AuthScope::Refresh),
            "com.atproto.appPass" => Ok(AuthScope::AppPass),
            "com.atproto.appPassPrivileged" => Ok(AuthScope::AppPassPrivileged),
            "com.atproto.signupQueued" => Ok(AuthScope::SignupQueued),
            _ => bail!("Invalid AuthScope: `{scope:?}` is not a valid auth scope"),
        }
    }
}

pub enum RoleStatus {
    Valid,
    Invalid,
    Missing,
}

#[derive(Clone)]
pub struct Credentials {
    pub r#type: String,
    pub did: Option<String>,
    pub scope: Option<AuthScope>,
    /// Raw OAuth scope string (space-separated) for tokens issued by the OAuth provider.
    /// `None` for legacy session JWTs. When set, per-method scope checks use
    /// `rsky_oauth_scopes::scope_permits_xrpc` rather than the coarse `AuthScope` enum.
    pub oauth_scope: Option<String>,
    pub audience: Option<String>,
    pub token_id: Option<String>,
    pub aud: Option<String>,
    pub iss: Option<String>,
    pub is_privileged: Option<bool>,
}

#[derive(Clone)]
pub struct AccessOutput {
    pub credentials: Option<Credentials>,
    pub artifacts: Option<String>,
}

pub struct ValidatedBearer {
    pub did: String,
    pub scope: AuthScope,
    pub token: String,
    pub payload: JwtPayload,
    pub audience: Option<String>,
}

pub struct AuthVerifierDids {
    pub pds: String,
    pub entryway: Option<String>,
    pub mod_service: Option<String>,
}

pub struct ServiceJwtOpts {
    pub aud: Option<String>,
    pub iss: Option<Vec<String>>,
}

pub struct ValidateAccessTokenOpts {
    pub check_takedown: Option<bool>,
    pub check_deactivated: Option<bool>,
}

pub struct VerifiedServiceJwt {
    pub aud: String,
    pub iss: String,
}

pub struct BasicAuth {
    pub username: String,
    pub password: String,
}

#[derive(Clone)]
pub struct JwtPayload {
    pub scope: AuthScope,
    pub sub: Option<String>,
    pub aud: Option<Audiences>,
    pub exp: Option<Duration>,
    pub iat: Option<Duration>,
    pub jti: Option<String>,
}

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("BadJwt: `{0}`")]
    BadJwt(String),
    #[error("BadJwtAudience: `{0}`")]
    BadJwtAudience(String),
    #[error("UntrustedIss: `{0}`")]
    UntrustedIss(String),
    #[error("AuthRequired: `{0}`")]
    AuthRequired(String),
    #[error("AccountNotFound: `{0}`")]
    AccountNotFound(String),
    #[error("AccountTakedown: `{0}`")]
    AccountTakedown(String),
    #[error("AccountDeactivated: `{0}`")]
    AccountDeactivated(String),
    #[error("InternalServerError: `{0}`")]
    InternalServerError(String),
}

// verifier guards

pub struct Refresh {
    pub access: AccessOutput,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Refresh {
    type Error = AuthError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let mut options = VerificationOptions::default();
        options.allowed_audiences = Some(HashSet::from_strings(&[
            env::var("PDS_SERVICE_DID").unwrap()
        ]));
        let ValidatedBearer {
            did,
            scope,
            token,
            payload,
            audience,
        } = match validate_bearer_token(req, vec![AuthScope::Refresh], Some(options)) {
            Ok(result) => {
                let payload = result.payload.clone();
                match payload.jti {
                    Some(_) => result,
                    None => {
                        let error =
                            AuthError::BadJwt("Unexpected missing refresh token id".to_owned());
                        req.local_cache(|| Some(ApiError::InvalidRequest(error.to_string())));
                        return Outcome::Error((Status::BadRequest, error));
                    }
                }
            }
            Err(error) => {
                let error = AuthError::BadJwt(error.to_string());
                req.local_cache(|| Some(ApiError::InvalidRequest(error.to_string())));
                return Outcome::Error((Status::BadRequest, error));
            }
        };
        Outcome::Success(Refresh {
            access: AccessOutput {
                credentials: Some(Credentials {
                    r#type: "refresh".to_string(),
                    did: Some(did),
                    scope: Some(scope),
                    oauth_scope: None,
                    audience,
                    token_id: payload.jti,
                    aud: None,
                    iss: None,
                    is_privileged: None,
                }),
                artifacts: Some(token),
            },
        })
    }
}

pub async fn access_check<'r>(
    req: &'r Request<'_>,
    scopes: Vec<AuthScope>,
    opts: Option<ValidateAccessTokenOpts>,
) -> Outcome<AccessOutput, AuthError> {
    match validate_access_token(req, scopes, opts).await {
        Ok(access) => Outcome::Success(access),
        Err(error) => match error.downcast_ref() {
            Some(AuthError::AccountDeactivated(error)) => Outcome::Error((
                Status::BadRequest,
                AuthError::AccountDeactivated(error.to_string()),
            )),
            Some(AuthError::AccountNotFound(error)) => Outcome::Error((
                Status::BadRequest,
                AuthError::AccountNotFound(error.to_string()),
            )),
            Some(AuthError::AccountTakedown(error)) => Outcome::Error((
                Status::BadRequest,
                AuthError::AccountTakedown(error.to_string()),
            )),
            _ => Outcome::Error((Status::BadRequest, AuthError::BadJwt(error.to_string()))),
        },
    }
}

pub struct AccessFullImport {
    pub access: AccessOutput,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AccessFullImport {
    type Error = AuthError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let opts = ValidateAccessTokenOpts {
            check_takedown: Some(true),
            check_deactivated: Some(false),
        };
        match access_check(req, vec![AuthScope::Access], Some(opts)).await {
            Outcome::Success(access) => Outcome::Success(AccessFullImport { access }),
            Outcome::Error(error) => Outcome::Error(error),
            Outcome::Forward(_) => panic!("Outcome::Forward returned"),
        }
    }
}

pub struct AccessFull {
    pub access: AccessOutput,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AccessFull {
    type Error = AuthError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match access_check(req, vec![AuthScope::Access], None).await {
            Outcome::Success(access) => Outcome::Success(AccessFull { access }),
            Outcome::Error(error) => {
                req.local_cache(|| Some(ApiError::InvalidRequest(error.1.to_string())));
                Outcome::Error(error)
            }
            Outcome::Forward(_) => panic!("Outcome::Forward returned"),
        }
    }
}

pub struct AccessPrivileged {
    pub access: AccessOutput,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AccessPrivileged {
    type Error = AuthError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match access_check(
            req,
            vec![AuthScope::Access, AuthScope::AppPassPrivileged],
            None,
        )
        .await
        {
            Outcome::Success(access) => Outcome::Success(Self { access }),
            Outcome::Error(error) => {
                req.local_cache(|| Some(ApiError::InvalidRequest(error.1.to_string())));
                Outcome::Error(error)
            }
            Outcome::Forward(_) => panic!("Outcome::Forward returned"),
        }
    }
}

pub struct AccessStandard {
    pub access: AccessOutput,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AccessStandard {
    type Error = AuthError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match access_check(
            req,
            vec![
                AuthScope::Access,
                AuthScope::AppPass,
                AuthScope::AppPassPrivileged,
            ],
            None,
        )
        .await
        {
            Outcome::Success(access) => Outcome::Success(AccessStandard { access }),
            Outcome::Error(error) => {
                req.local_cache(|| Some(ApiError::InvalidRequest(error.1.to_string())));
                Outcome::Error(error)
            }
            Outcome::Forward(_) => panic!("Outcome::Forward returned"),
        }
    }
}

#[derive(Clone)]
pub struct AccessStandardIncludeChecks {
    pub access: AccessOutput,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AccessStandardIncludeChecks {
    type Error = AuthError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match access_check(
            req,
            vec![
                AuthScope::Access,
                AuthScope::AppPass,
                AuthScope::AppPassPrivileged,
            ],
            Some(ValidateAccessTokenOpts {
                check_deactivated: Some(true),
                check_takedown: Some(true),
            }),
        )
        .await
        {
            Outcome::Success(access) => Outcome::Success(AccessStandardIncludeChecks { access }),
            Outcome::Error(error) => {
                req.local_cache(|| Some(ApiError::InvalidRequest(error.1.to_string())));
                Outcome::Error(error)
            }
            Outcome::Forward(_) => panic!("Outcome::Forward returned"),
        }
    }
}

#[derive(Clone)]
pub struct AccessStandardCheckTakedown {
    pub access: AccessOutput,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AccessStandardCheckTakedown {
    type Error = AuthError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match access_check(
            req,
            vec![
                AuthScope::Access,
                AuthScope::AppPass,
                AuthScope::AppPassPrivileged,
            ],
            Some(ValidateAccessTokenOpts {
                check_deactivated: None,
                check_takedown: Some(true),
            }),
        )
        .await
        {
            Outcome::Success(access) => Outcome::Success(AccessStandardCheckTakedown { access }),
            Outcome::Error(error) => {
                req.local_cache(|| Some(ApiError::InvalidRequest(error.1.to_string())));
                Outcome::Error(error)
            }
            Outcome::Forward(_) => panic!("Outcome::Forward returned"),
        }
    }
}

pub struct AccessStandardSignupQueued {
    pub access: AccessOutput,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AccessStandardSignupQueued {
    type Error = AuthError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match access_check(
            req,
            vec![
                AuthScope::Access,
                AuthScope::AppPass,
                AuthScope::AppPassPrivileged,
                AuthScope::SignupQueued,
            ],
            None,
        )
        .await
        {
            Outcome::Success(access) => Outcome::Success(AccessStandardSignupQueued { access }),
            Outcome::Error(error) => {
                req.local_cache(|| Some(ApiError::InvalidRequest(error.1.to_string())));
                Outcome::Error(error)
            }
            Outcome::Forward(_) => panic!("Outcome::Forward returned"),
        }
    }
}

pub struct RevokeRefreshToken {
    pub id: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RevokeRefreshToken {
    type Error = AuthError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let mut options = VerificationOptions::default();
        options.max_validity = Some(Duration::from_secs(INFINITY));
        match validate_bearer_token(req, vec![AuthScope::Refresh], Some(options)) {
            Ok(result) => match result.payload.jti {
                Some(jti) => Outcome::Success(RevokeRefreshToken { id: jti }),
                None => {
                    let error = AuthError::BadJwt("Unexpected missing refresh token id".to_owned());
                    req.local_cache(|| Some(ApiError::InvalidRequest(error.to_string())));
                    Outcome::Error((Status::BadRequest, error))
                }
            },
            Err(error) => {
                req.local_cache(|| Some(ApiError::InvalidRequest(error.to_string())));
                Outcome::Error((Status::BadRequest, AuthError::BadJwt(error.to_string())))
            }
        }
    }
}

pub struct UserDidAuth {
    pub access: AccessOutput,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserDidAuth {
    type Error = AuthError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let id_resolver = req.guard::<&State<SharedIdResolver>>().await.unwrap();
        match verify_service_jwt(
            req,
            id_resolver,
            ServiceJwtOpts {
                aud: Some(env::var("PDS_SERVICE_DID").unwrap()),
                iss: None,
            },
        )
        .await
        {
            Ok(payload) => Outcome::Success(UserDidAuth {
                access: AccessOutput {
                    credentials: Some(Credentials {
                        r#type: "user_did".to_string(),
                        did: None,
                        scope: None,
                        oauth_scope: None,
                        audience: None,
                        token_id: None,
                        aud: Some(payload.aud),
                        iss: Some(payload.iss),
                        is_privileged: None,
                    }),
                    artifacts: None,
                },
            }),
            Err(error) => {
                req.local_cache(|| {
                    Some(ApiError::InvalidRequest(
                        AuthError::BadJwt(error.to_string()).to_string(),
                    ))
                });
                Outcome::Error((Status::BadRequest, AuthError::BadJwt(error.to_string())))
            }
        }
    }
}

pub struct UserDidAuthOptional {
    pub access: Option<AccessOutput>,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserDidAuthOptional {
    type Error = AuthError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        if is_bearer_token(req) {
            match UserDidAuth::from_request(req).await {
                Outcome::Success(output) => Outcome::Success(UserDidAuthOptional {
                    access: Some(output.access),
                }),
                Outcome::Error(err) => {
                    req.local_cache(|| Some(ApiError::InvalidRequest(err.1.to_string())));
                    Outcome::Error(err)
                }
                _ => panic!("Unexpected outcome during UserDidAuthOptional"),
            }
        } else {
            Outcome::Success(UserDidAuthOptional { access: None })
        }
    }
}

pub struct ModService {
    pub access: AccessOutput,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ModService {
    type Error = AuthError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        if let Some(mod_service_did) = env_str("PDS_MOD_SERVICE_DID") {
            let id_resolver = req.guard::<&State<SharedIdResolver>>().await.unwrap();
            match verify_service_jwt(
                req,
                id_resolver,
                ServiceJwtOpts {
                    aud: None,
                    iss: Some(vec![
                        mod_service_did.clone(),
                        format!("{mod_service_did}#atproto_labeler"),
                    ]),
                },
            )
            .await
            {
                Ok(payload)
                    if Some(payload.aud.clone()) != env_str("PDS_SERVICE_DID")
                        && (env_str("PDS_ENTRYWAY_DID").is_none()
                            || Some(payload.aud.clone()) != env_str("PDS_ENTRYWAY_DID")) =>
                {
                    let error = AuthError::BadJwtAudience(
                        "jwt audience does not match service did".to_string(),
                    );
                    req.local_cache(|| Some(ApiError::InvalidRequest(error.to_string())));
                    Outcome::Error((Status::BadRequest, error))
                }
                Ok(payload) => Outcome::Success(ModService {
                    access: AccessOutput {
                        credentials: Some(Credentials {
                            r#type: "mod_service".to_string(),
                            did: None,
                            scope: None,
                            oauth_scope: None,
                            audience: None,
                            token_id: None,
                            aud: Some(payload.aud),
                            iss: Some(payload.iss),
                            is_privileged: None,
                        }),
                        artifacts: None,
                    },
                }),
                Err(error) => {
                    let error = AuthError::BadJwt(error.to_string());
                    req.local_cache(|| Some(ApiError::InvalidRequest(error.to_string())));
                    Outcome::Error((Status::BadRequest, AuthError::BadJwt(error.to_string())))
                }
            }
        } else {
            let error = AuthError::UntrustedIss("Untrusted issuer".to_string());
            req.local_cache(|| Some(ApiError::InvalidRequest(error.to_string())));
            Outcome::Error((Status::BadRequest, error))
        }
    }
}

pub struct Moderator {
    pub access: AccessOutput,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Moderator {
    type Error = AuthError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        if is_bearer_token(req) {
            match ModService::from_request(req).await {
                Outcome::Success(output) => Outcome::Success(Moderator {
                    access: output.access,
                }),
                Outcome::Error(err) => {
                    req.local_cache(|| Some(ApiError::InvalidRequest(err.1.to_string())));
                    Outcome::Error(err)
                }
                _ => panic!("Unexpected outcome during Moderator"),
            }
        } else {
            match AdminToken::from_request(req).await {
                Outcome::Success(output) => Outcome::Success(Moderator {
                    access: output.access,
                }),
                Outcome::Error(err) => {
                    req.local_cache(|| Some(ApiError::InvalidRequest(err.1.to_string())));
                    Outcome::Error(err)
                }
                _ => panic!("Unexpected outcome during Moderator"),
            }
        }
    }
}

pub struct AdminToken {
    pub access: AccessOutput,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminToken {
    type Error = AuthError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let auth_header: &str = req.headers().get_one("Authorization").unwrap_or("");
        match parse_basic_auth(auth_header) {
            None => {
                req.local_cache(|| {
                    Some(ApiError::AuthRequiredError(
                        "AuthRequired: credentials required".to_string(),
                    ))
                });
                Outcome::Error((
                    Status::Unauthorized,
                    AuthError::AuthRequired("AuthMissing".to_string()),
                ))
            }
            Some(parsed) => {
                let BasicAuth { username, password } = parsed;

                if username != "admin" || password != env::var("PDS_ADMIN_PASS").unwrap() {
                    let error = AuthError::AuthRequired("BadAuth".to_string());
                    req.local_cache(|| Some(ApiError::InvalidRequest(error.to_string())));
                    Outcome::Error((Status::BadRequest, error))
                } else {
                    Outcome::Success(AdminToken {
                        access: AccessOutput {
                            credentials: Some(Credentials {
                                r#type: "admin_token".to_string(),
                                did: None,
                                scope: None,
                                oauth_scope: None,
                                audience: None,
                                token_id: None,
                                aud: None,
                                iss: None,
                                is_privileged: None,
                            }),
                            artifacts: None,
                        },
                    })
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct OptionalAccessOrAdminToken {
    pub access: Option<AccessOutput>,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for OptionalAccessOrAdminToken {
    type Error = AuthError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        if is_bearer_token(req) {
            match AccessFull::from_request(req).await {
                Outcome::Success(output) => Outcome::Success(OptionalAccessOrAdminToken {
                    access: Some(output.access),
                }),
                Outcome::Error(err) => {
                    req.local_cache(|| Some(ApiError::InvalidRequest(err.1.to_string())));
                    Outcome::Error(err)
                }
                _ => panic!("Unexpected outcome during OptionalAccessOrAdminToken"),
            }
        } else if is_basic_token(req) {
            match AdminToken::from_request(req).await {
                Outcome::Success(output) => Outcome::Success(OptionalAccessOrAdminToken {
                    access: Some(output.access),
                }),
                Outcome::Error(err) => {
                    req.local_cache(|| Some(ApiError::InvalidRequest(err.1.to_string())));
                    Outcome::Error(err)
                }
                _ => panic!("Unexpected outcome during OptionalAccessOrAdminToken"),
            }
        } else {
            Outcome::Success(OptionalAccessOrAdminToken { access: None })
        }
    }
}

pub async fn validate_bearer_access_token<'r>(
    request: &'r Request<'_>,
    scopes: Vec<AuthScope>,
) -> Result<AccessOutput> {
    let mut options = VerificationOptions::default();
    options.allowed_audiences = Some(HashSet::from_strings(&[
        env::var("PDS_SERVICE_DID").unwrap()
    ]));
    let ValidatedBearer {
        did,
        scope,
        token,
        audience,
        ..
    } = validate_bearer_token(request, scopes, Some(options))?;
    let is_privileged = vec![AuthScope::Access, AuthScope::AppPassPrivileged].contains(&scope);
    Ok(AccessOutput {
        credentials: Some(Credentials {
            r#type: "access".to_string(),
            did: Some(did),
            scope: Some(scope),
            oauth_scope: None,
            audience,
            token_id: None,
            aud: None,
            iss: None,
            is_privileged: Some(is_privileged),
        }),
        artifacts: Some(token),
    })
}

pub fn validate_bearer_token<'r>(
    request: &'r Request<'_>,
    scopes: Vec<AuthScope>,
    verify_options: Option<VerificationOptions>,
) -> Result<ValidatedBearer> {
    let token = bearer_token_from_req(request)?;
    if let Some(token) = token {
        let payload = verify_jwt(&token, verify_options)?;
        let JwtPayload {
            sub, aud, scope, ..
        } = payload.clone();
        let sub = sub.unwrap();
        let aud = aud.unwrap();
        if !sub.starts_with("did:") {
            bail!("Malformed token")
        }
        if let Audiences::AsString(aud) = aud {
            if !aud.starts_with("did:") {
                bail!("Malformed token")
            }
            if scopes.len() > 0 && !scopes.contains(&scope) {
                bail!("Bad token scope")
                /*{
                    "error": "InvalidToken",
                    "message": "Bad token scope"
                }*/
            }
            Ok(ValidatedBearer {
                did: sub,
                scope,
                audience: Some(aud),
                token,
                payload,
            })
        } else {
            bail!("Malformed token")
        }
    } else {
        bail!("AuthMissing")
    }
}

// @TODO: Implement DPop/OAuth
pub async fn validate_access_token<'r>(
    request: &'r Request<'_>,
    scopes: Vec<AuthScope>,
    opts: Option<ValidateAccessTokenOpts>,
) -> Result<AccessOutput> {
    // --- Token-type discriminator ---
    // OAuth access tokens issued by rsky-pds (p3-c009) carry a `client_id` claim
    // that is absent from legacy session JWTs.  We detect this by doing a cheap
    // base64url decode of the payload section before full verification so we can
    // branch without re-implementing the signature check.
    let raw_token = bearer_token_from_req(request)?;
    if let Some(ref tok) = raw_token {
        if let Some(oauth_scope_str) = try_extract_oauth_scope(tok) {
            // OAuth path: validate signature with the same key as legacy JWTs
            // (our OAuth tokens are signed with PDS_JWT_KEYPAIR in p3-c009).
            let mut options = VerificationOptions::default();
            options.allowed_audiences = Some(HashSet::from_strings(&[
                env::var("PDS_SERVICE_DID").unwrap()
            ]));
            match PDS_JWT_KEYPAIR
                .public_key()
                .verify_token::<serde_json::Value>(tok, Some(options))
            {
                Err(_) => bail!("OAuthTokenInvalid: invalid OAuth access token signature"),
                Ok(claims) => {
                    let did = match claims.subject {
                        Some(sub) => sub,
                        None => bail!("OAuthTokenInvalid: missing sub claim"),
                    };
                    let ValidateAccessTokenOpts {
                        check_takedown,
                        check_deactivated,
                    } = opts.unwrap_or_else(|| ValidateAccessTokenOpts {
                        check_takedown: Some(false),
                        check_deactivated: Some(false),
                    });
                    let check_takedown = check_takedown.unwrap_or(false);
                    let check_deactivated = check_deactivated.unwrap_or(false);
                    if check_takedown || check_deactivated {
                        let account_manager = match request.guard::<AccountManager>().await {
                            Outcome::Success(am) => am,
                            _ => {
                                return Err(anyhow::Error::new(AuthError::InternalServerError(
                                    "Unexpected Error Occurred".to_string(),
                                )))
                            }
                        };
                        let found: ActorAccount = match account_manager
                            .get_account(
                                &did,
                                Some(AvailabilityFlags {
                                    include_deactivated: Some(true),
                                    include_taken_down: Some(true),
                                }),
                            )
                            .await
                        {
                            Ok(Some(found)) => found,
                            _ => {
                                return Err(anyhow::Error::new(AuthError::AccountNotFound(
                                    "Account not found".to_string(),
                                )))
                            }
                        };
                        if check_takedown && found.takedown_ref.is_some() {
                            return Err(anyhow::Error::new(AuthError::AccountTakedown(
                                "Account has been taken down".to_string(),
                            )));
                        }
                        if check_deactivated && found.deactivated_at.is_some() {
                            return Err(anyhow::Error::new(AuthError::AccountDeactivated(
                                "Account is deactivated".to_string(),
                            )));
                        }
                    }
                    return Ok(AccessOutput {
                        credentials: Some(Credentials {
                            r#type: "access".to_string(),
                            did: Some(did),
                            scope: Some(AuthScope::Access),
                            oauth_scope: Some(oauth_scope_str),
                            audience: None,
                            token_id: claims.jwt_id,
                            aud: None,
                            iss: None,
                            is_privileged: None,
                        }),
                        artifacts: Some(tok.clone()),
                    });
                }
            }
        }
    }

    // --- Legacy session JWT path ---
    let mut options = VerificationOptions::default();
    options.allowed_audiences = Some(HashSet::from_strings(&[
        env::var("PDS_SERVICE_DID").unwrap()
    ]));
    let ValidatedBearer {
        did,
        scope,
        token,
        audience,
        ..
    } = validate_bearer_token(request, scopes, Some(options))?;
    let ValidateAccessTokenOpts {
        check_takedown,
        check_deactivated,
    } = opts.unwrap_or_else(|| ValidateAccessTokenOpts {
        check_takedown: Some(false),
        check_deactivated: Some(false),
    });
    let check_takedown = check_takedown.unwrap_or(false);
    let check_deactivated = check_deactivated.unwrap_or(false);

    let account_manager = match request
        .guard::<AccountManager>()
        .await
        .map(|account_manager| account_manager)
    {
        Outcome::Success(account_manager) => account_manager,
        Outcome::Error(_) => {
            return Err(anyhow::Error::new(AuthError::InternalServerError(
                "Unexpected Error Occurred".to_string(),
            )))
        }
        Outcome::Forward(_) => {
            return Err(anyhow::Error::new(AuthError::InternalServerError(
                "Unexpected Error Occurred".to_string(),
            )))
        }
    };
    if check_takedown || check_deactivated {
        let found: ActorAccount = match account_manager
            .get_account(
                &did,
                Some(AvailabilityFlags {
                    include_deactivated: Some(true),
                    include_taken_down: Some(true),
                }),
            )
            .await
        {
            Ok(Some(found)) => found,
            _ => {
                return Err(anyhow::Error::new(AuthError::AccountNotFound(
                    "Account not found".to_string(),
                )))
            }
        };
        if check_takedown && found.takedown_ref.is_some() {
            return Err(anyhow::Error::new(AuthError::AccountTakedown(
                "Account has been taken down".to_string(),
            )));
        }
        if check_deactivated && found.deactivated_at.is_some() {
            return Err(anyhow::Error::new(AuthError::AccountDeactivated(
                "Account is deactivated".to_string(),
            )));
        }
    }
    Ok(AccessOutput {
        credentials: Some(Credentials {
            r#type: "access".to_string(),
            did: Some(did),
            scope: Some(scope),
            oauth_scope: None,
            audience,
            token_id: None,
            aud: None,
            iss: None,
            is_privileged: None,
        }),
        artifacts: Some(token),
    })
}

pub async fn verify_service_jwt<'r>(
    request: &'r Request<'_>,
    id_resolver: &State<SharedIdResolver>,
    opts: ServiceJwtOpts,
) -> Result<VerifiedServiceJwt> {
    let get_signing_key = |iss: String, force_refresh: bool| -> Result<String> {
        match &opts.iss {
            Some(opts_iss) if opts_iss.contains(&iss) => bail!("UntrustedIss: Untrusted issuer"),
            _ => (),
        }
        let parts = iss.split("#").collect::<Vec<&str>>();
        if let (Some(did), Some(service_id)) = (parts.get(0), parts.get(1)) {
            let (did, service_id) = (did.to_string(), *service_id);
            let key_id = if service_id == "atproto_labeler" {
                "atproto_label"
            } else {
                "atproto"
            };
            let mut lock = futures::executor::block_on(id_resolver.id_resolver.write());
            let did_doc: Result<DidDocument> =
                futures::executor::block_on(lock.did.ensure_resolve(&did, Some(force_refresh)));
            let did_doc: DidDocument = match did_doc {
                Err(err) => bail!("could not resolve iss did: `{err}`"),
                Ok(res) => res,
            };
            match get_verification_material(&did_doc, &key_id.to_string()) {
                None => bail!("missing or bad key in did doc"),
                Some(parsed_key) => match get_did_key_from_multibase(parsed_key)? {
                    None => bail!("missing or bad key in did doc"),
                    Some(did_key) => Ok(did_key),
                },
            }
        } else {
            bail!("could not resolve iss did")
        }
    };

    match bearer_token_from_req(request)? {
        None => bail!("MissingJwt: missing jwt"),
        Some(jwt_str) => {
            let payload: ServiceJwtPayload =
                verify_service_jwt_server(jwt_str, opts.aud, get_signing_key).await?;
            Ok(VerifiedServiceJwt {
                iss: payload.iss,
                aud: payload.aud,
            })
        }
    }
}

pub fn is_user_or_admin(auth: AccessOutput, did: &String) -> bool {
    match auth.credentials {
        Some(credentials) if credentials.did == Some("admin_token".to_string()) => true,
        Some(credentials) => credentials.did == Some(did.to_string()),
        None => false,
    }
}

// HELPERS
// ---------

const BEARER: &str = "Bearer ";
const BASIC: &str = "Basic ";

pub fn is_bearer_token(request: &Request) -> bool {
    match request.headers().get_one("Authorization") {
        None => false,
        Some(auth_header) => auth_header.starts_with(BEARER),
    }
}

pub fn is_basic_token(request: &Request) -> bool {
    match request.headers().get_one("Authorization") {
        None => false,
        Some(auth_header) => auth_header.starts_with(BASIC),
    }
}

/// Cheaply detect whether a JWT is an OAuth access token (vs. a legacy session JWT)
/// by base64url-decoding its payload section and looking for a `client_id` claim.
/// OAuth tokens issued by rsky-pds (p3-c009) always carry `client_id`; legacy JWTs
/// do not.  Returns the raw scope string from the OAuth token, or `None` if not OAuth.
pub fn try_extract_oauth_scope(jwt: &str) -> Option<String> {
    let parts: Vec<&str> = jwt.splitn(3, '.').collect();
    let payload_b64 = parts.get(1)?;
    // base64url without padding
    let padding = match payload_b64.len() % 4 {
        0 => 0,
        n => 4 - n,
    };
    let padded = format!("{}{}", payload_b64, "=".repeat(padding));
    let bytes = base64url.decode(&padded).ok()?;
    let json: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    // Must have `client_id` to be considered an OAuth token
    json.get("client_id")?;
    // Extract the `scope` claim (space-separated string)
    let scope = json.get("scope")?.as_str()?.to_string();
    Some(scope)
}

pub fn bearer_token_from_req(request: &Request) -> Result<Option<String>> {
    match request.headers().get_one("authorization") {
        Some(header) if !header.starts_with("Bearer ") => Ok(None),
        Some(header) => {
            let slice = &header["Bearer ".len()..];
            Ok(Some(slice.to_string()))
        }
        None => Ok(None),
    }
}

pub fn verify_jwt(jwt: &str, verify_options: Option<VerificationOptions>) -> Result<JwtPayload> {
    let claims = PDS_JWT_KEYPAIR
        .public_key()
        .verify_token::<CustomClaimObj>(jwt, verify_options)?;

    Ok(JwtPayload {
        scope: AuthScope::from_str(&claims.custom.scope)?,
        sub: claims.subject,
        aud: claims.audiences,
        exp: claims.expires_at,
        iat: claims.issued_at,
        jti: claims.jwt_id,
    })
}

pub fn parse_basic_auth(token: &str) -> Option<BasicAuth> {
    if !token.starts_with(BASIC) {
        return None;
    }

    let b64 = &token[BASIC.len()..];
    let decoded: Vec<u8> = match base64pad.decode(b64) {
        Err(_) => return None,
        Ok(decoded) => decoded,
    };
    let parsed_str: &str = match str::from_utf8(&decoded) {
        Err(_) => return None,
        Ok(res) => res,
    };
    let parsed_parts = parsed_str.split(":").collect::<Vec<&str>>();

    match (parsed_parts.get(0), parsed_parts.get(1)) {
        (Some(username), Some(password)) => Some(BasicAuth {
            username: username.to_string(),
            password: password.to_string(),
        }),
        _ => None,
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// UAR / A2A Agent Auth
// ──────────────────────────────────────────────────────────────────────────────

/// Result of a verified agent service JWT.
///
/// The `agent_did` may be any DID method permitted by `DidCapability::AgentIdentity`
/// (`did:plc`, `did:key`). It is **never** a bare account DID with a PDS service —
/// the profile validator blocks `AccountIdentity`-only documents.
#[derive(Clone, Debug)]
pub struct AgentJwtContext {
    /// The issuer DID of the agent JWT.
    pub agent_did: String,
    /// The audience DID (must match `PDS_SERVICE_DID`).
    pub aud: String,
    /// The lexicon method the token is scoped to, if any.
    pub lxm: Option<String>,
}

/// Rocket request guard for A2A / UAR agent tokens.
///
/// Accepts service JWTs whose `iss` resolves as `DidCapability::AgentIdentity`
/// (did:plc or did:key). Rejects tokens from plain account DIDs that have an
/// `AtprotoPersonalDataServer` service (those must use the standard `AccessFull` path).
///
/// Adds zero overhead for non-agent routes: simply don't put this guard on them.
pub struct AgentAuth {
    pub agent: AgentJwtContext,
}

/// Verify a service JWT carrying an agent DID as `iss`.
///
/// Uses `CompositeDidResolver` with `AgentIdentity` validation so that:
/// - `did:plc` agents that do NOT have an `AtprotoPersonalDataServer` service pass
/// - `did:key` agents always pass (offline, no PDS service required)
/// - bare account DIDs (did:plc with `AtprotoPersonalDataServer`) are rejected by
///   the `AccountIdentity` profile constraint being absent — but still resolve fine;
///   they fail only when the caller checks `AgentIdentity` and the doc doesn't fit
///
/// Returns `Err` on any validation failure; the caller should map to 401.
pub async fn verify_agent_service_jwt(
    jwt_str: &str,
    expected_aud: &str,
    composite: &CompositeDidResolver,
) -> Result<AgentJwtContext> {
    // Decode payload without verification first to extract `iss` and `aud`.
    let parts: Vec<&str> = jwt_str.splitn(3, '.').collect();
    if parts.len() != 3 {
        bail!("AgentJwt: malformed JWT");
    }
    let payload_b64 = parts[1];
    let payload_bytes = base64_url::decode(payload_b64)
        .map_err(|_| anyhow::anyhow!("AgentJwt: invalid base64 payload"))?;
    let payload: crate::account_manager::helpers::auth::ServiceJwtPayload =
        serde_json::from_slice(&payload_bytes)
            .map_err(|e| anyhow::anyhow!("AgentJwt: payload parse error: {e}"))?;

    let iss = payload.iss.clone();
    let aud = payload.aud.clone();

    if aud != expected_aud {
        bail!("AgentJwt: audience mismatch: got {aud}, expected {expected_aud}");
    }
    let resolution = composite
        .resolve_for_capability(&iss, &DidCapability::AgentIdentity)
        .await
        .map_err(|e: DidError| {
            anyhow::anyhow!("AgentJwt: issuer DID not valid for AgentIdentity: {e}")
        })?;

    // Fetch the signing key from the DID document.
    // For did:key the key is inline; for did:plc it is the #atproto verification method.
    let signing_key = if iss.starts_with("did:key:") {
        // The multibase key is the did:key identifier itself (strip prefix).
        let multibase = iss.trim_start_matches("did:key:");
        use rsky_identity::did::atproto_data::{get_did_key_from_multibase, VerificationMaterial};
        get_did_key_from_multibase(VerificationMaterial {
            r#type: "Multikey".to_string(),
            public_key_multibase: multibase.to_string(),
        })
        .map_err(|e| anyhow::anyhow!("AgentJwt: did:key material parse error: {e}"))?
        .ok_or_else(|| anyhow::anyhow!("AgentJwt: did:key has no supported key type"))?
    } else {
        // did:plc or other method: look for #atproto in verificationMethod.
        get_verification_material(&resolution.document, &"atproto".to_string())
            .and_then(|mat| get_did_key_from_multibase(mat).ok().and_then(|k| k))
            .ok_or_else(|| anyhow::anyhow!("AgentJwt: no #atproto key in issuer DID document"))?
    };

    // Verify the JWT signature using the resolved key.
    crate::xrpc_server::auth::verify_jwt(
        jwt_str.to_string(),
        Some(expected_aud.to_string()),
        |_iss: String, _force_refresh: bool| Ok(signing_key.clone()),
    )
    .await
    .map_err(|e| anyhow::anyhow!("AgentJwt: signature verification failed: {e}"))?;

    Ok(AgentJwtContext {
        agent_did: iss,
        aud,
        lxm: payload.lxm,
    })
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AgentAuth {
    type Error = ApiError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let jwt_str = match bearer_token_from_req(request) {
            Ok(Some(t)) => t,
            _ => {
                return Outcome::Error((
                    Status::Unauthorized,
                    ApiError::AuthRequiredError("missing bearer token".to_string()),
                ))
            }
        };

        let service_did = match env_str("PDS_SERVICE_DID") {
            Some(d) => d,
            None => return Outcome::Error((Status::InternalServerError, ApiError::RuntimeError)),
        };

        let composite = request
            .guard::<&State<SharedCompositeResolver>>()
            .await
            .unwrap();

        match verify_agent_service_jwt(&jwt_str, &service_did, &composite.resolver).await {
            Ok(agent) => Outcome::Success(AgentAuth { agent }),
            Err(e) => {
                tracing::warn!("AgentAuth rejected: {e}");
                Outcome::Error((
                    Status::Unauthorized,
                    ApiError::AuthRequiredError(e.to_string()),
                ))
            }
        }
    }
}
