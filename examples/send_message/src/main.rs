use std::collections::HashMap;

use rs_firebase_admin_sdk::{
    fcm::{Apns, FirebaseFcmService, Message, Notification, SendMessageRequest},
    App, CustomServiceAccount,
};

#[tokio::main]
async fn main() {
    // Live Firebase App
    let gcp_service_account = CustomServiceAccount::from_file(
        // Read JSON contents for service account key from environment
        &std::env::var("SERVICE_ACCOUNT_KEY").expect("SERVICE_ACCOUNT_KEY not set"),
    )
    .unwrap();

    let test_token = std::env::var("FCM_TEST_TOKEN").expect("FCM_TEST_TOKEN not set");

    let live_app = App::live(gcp_service_account.into()).await.unwrap();

    let live_fcm_admin = live_app.fcm();

    let data = HashMap::new();

    live_fcm_admin
        .send_message(SendMessageRequest {
            validate_only: false,
            message: Message {
                data,
                notification: Notification {
                    title: "test".into(),
                    body: "test".into(),
                },
                apns: Apns::default(),
                token: test_token.into(),
            },
        })
        .await
        .unwrap();
}
