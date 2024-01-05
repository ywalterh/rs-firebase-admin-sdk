pub mod api_uri;
pub mod auth;
pub mod client;
pub mod credentials;
pub mod fcm;
pub mod util;

use auth::{
    token::{
        cache::{HttpCache, PubKeys},
        error::TokenVerificationError,
        EmulatedTokenVerifier, LiveTokenVerifier, GOOGLE_COOKIE_PUB_KEY_URI, GOOGLE_PUB_KEY_URI,
    },
    FirebaseAuth,
};
use client::{build_https_client, HyperApiClient, HyperClient};
use credentials::emulator::EmulatorCredentials;
pub use credentials::{error::CredentialsError, gcp::GcpCredentials};
use error_stack::{Report, ResultExt};
use fcm::FirebaseFcm;
pub use gcp_auth::CustomServiceAccount;
use http::uri::Authority;
use std::sync::Arc;

/// Default Firebase Auth admin manager
pub type LiveAuthAdmin = FirebaseAuth<HyperApiClient<GcpCredentials>>;
/// Default Firebase Auth Emulator admin manager
pub type EmulatorAuthAdmin = FirebaseAuth<HyperApiClient<EmulatorCredentials>>;
/// Default Firebase Auth admin manager
pub type LiveFcmAdmin = FirebaseFcm<HyperApiClient<GcpCredentials>>;

/// Base privileged manager for Firebase
pub struct App<CredentialsT> {
    credentials: Arc<CredentialsT>,
    project_id: String,
}

impl App<EmulatorCredentials> {
    /// Firebase app backend by emulator
    pub fn emulated(project_id: String) -> Self {
        Self {
            credentials: Arc::new(EmulatorCredentials {}),
            project_id,
        }
    }

    /// Firebase authentication manager for emulator
    pub fn auth(&self, emulator_auth: Authority) -> EmulatorAuthAdmin {
        let client = HyperApiClient::new(self.credentials.clone());

        FirebaseAuth::emulated(emulator_auth, &self.project_id, client)
    }

    /// OIDC token verifier for emulator
    pub fn id_token_verifier(&self) -> EmulatedTokenVerifier {
        EmulatedTokenVerifier::new(self.project_id.clone())
    }
}

impl App<GcpCredentials> {
    /// Create instance of Firebase app for live project
    pub async fn live(credentials: GcpCredentials) -> Result<Self, Report<CredentialsError>> {
        Self::live_shared(Arc::new(credentials)).await
    }

    pub async fn live_shared(
        credentials: Arc<GcpCredentials>,
    ) -> Result<Self, Report<CredentialsError>> {
        let project_id = credentials
            .project_id()
            .await
            .change_context(CredentialsError::Internal)?;

        Ok(Self {
            credentials,
            project_id,
        })
    }

    /// Create Firebase authentication manager
    pub fn auth(&self) -> LiveAuthAdmin {
        let client = HyperApiClient::new(self.credentials.clone());

        FirebaseAuth::live(&self.project_id, client)
    }

    /// Create Firebase FCM manager
    pub fn fcm(&self) -> LiveFcmAdmin {
        let client = HyperApiClient::new(self.credentials.clone());

        LiveFcmAdmin::live(&self.project_id, client)
    }

    /// Create OIDC token verifier
    pub async fn id_token_verifier(
        &self,
    ) -> Result<LiveTokenVerifier<HttpCache<HyperClient, PubKeys>>, Report<TokenVerificationError>>
    {
        let cache_client = HttpCache::new(
            build_https_client(),
            GOOGLE_PUB_KEY_URI
                .parse()
                .map_err(error_stack::Report::new)
                .change_context(TokenVerificationError::FailedGettingKeys)?,
        )
        .await
        .change_context(TokenVerificationError::FailedGettingKeys)?;

        LiveTokenVerifier::new_id_verifier(self.project_id.clone(), cache_client)
    }

    /// Create cookie token verifier
    pub async fn cookie_token_verifier(
        &self,
    ) -> Result<LiveTokenVerifier<HttpCache<HyperClient, PubKeys>>, Report<TokenVerificationError>>
    {
        let cache_client = HttpCache::new(
            build_https_client(),
            GOOGLE_COOKIE_PUB_KEY_URI
                .parse()
                .map_err(error_stack::Report::new)
                .change_context(TokenVerificationError::FailedGettingKeys)?,
        )
        .await
        .change_context(TokenVerificationError::FailedGettingKeys)?;

        LiveTokenVerifier::new_cookie_verifier(self.project_id.clone(), cache_client)
    }
}
