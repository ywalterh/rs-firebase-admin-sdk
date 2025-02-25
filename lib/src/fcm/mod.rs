use async_trait::async_trait;
use error_stack::{Report, ResultExt};
use http::uri::Scheme;
use http::Method;
use serde::{Deserialize, Serialize};

use crate::api_uri::{ApiUriBuilder, FirebaseFcmRestApi};
use crate::client::error::ApiClientError;
use crate::client::ApiHttpClient;

#[derive(Serialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageRequest {
    pub message: Message,
    pub validate_only: bool,
}

#[derive(Serialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub notification: Notification,
    pub apns: Apns,
    pub token: String,
}

#[derive(Serialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Notification {
    pub title: String,
    pub body: String,
}

#[derive(Serialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Apns {
    pub payload: ApnsPayload,
}

#[derive(Serialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ApnsPayload {
    pub aps: Aps,
}

#[derive(Serialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Aps {
    pub sound: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NewMessageResponse {}

const FIREBASE_FCM_REST_AUTHORITY: &str = "fcm.googleapis.com";

const FIREBASE_FCM_SCOPES: [&str; 2] = [
    "https://www.googleapis.com/auth/cloud-platform",
    "https://www.googleapis.com/auth/firebase.messaging",
];

pub struct FirebaseFcm<ApiHttpClientT> {
    client: ApiHttpClientT,
    fcm_uri_builder: ApiUriBuilder,
}

impl<ApiHttpClientT> FirebaseFcm<ApiHttpClientT>
where
    ApiHttpClientT: ApiHttpClient + Send + Sync,
{
    /// Create Firebase FCM manager for live project
    pub fn live(project_id: &str, client: ApiHttpClientT) -> Self {
        Self {
            client,
            fcm_uri_builder: ApiUriBuilder::new(
                Scheme::HTTPS,
                FIREBASE_FCM_REST_AUTHORITY
                    .parse()
                    .expect("Failed parsing auth service authority"),
                Some(format!("/v1/projects/{project_id}")),
            ),
        }
    }
}

impl<ApiHttpClientT> FirebaseFcmService<ApiHttpClientT> for FirebaseFcm<ApiHttpClientT>
where
    ApiHttpClientT: ApiHttpClient + Send + Sync,
{
    fn get_client(&self) -> &ApiHttpClientT {
        &self.client
    }

    fn get_auth_uri_builder(&self) -> &ApiUriBuilder {
        &self.fcm_uri_builder
    }
}

#[async_trait]
pub trait FirebaseFcmService<ApiHttpClientT>
where
    Self: Send + Sync,
    ApiHttpClientT: ApiHttpClient + Send + Sync,
{
    fn get_client(&self) -> &ApiHttpClientT;
    fn get_auth_uri_builder(&self) -> &ApiUriBuilder;

    async fn send_message(
        &self,
        request: SendMessageRequest,
    ) -> Result<NewMessageResponse, Report<ApiClientError>> {
        let client = self.get_client();
        let uri_builder = self.get_auth_uri_builder();

        let response: NewMessageResponse = client
            .send_request_body(
                uri_builder
                    .build(FirebaseFcmRestApi::SendMessage)
                    .change_context(ApiClientError::FailedToSendRequest)?,
                Method::POST,
                request,
                &FIREBASE_FCM_SCOPES,
            )
            .await?;

        Ok(response)
    }
}
