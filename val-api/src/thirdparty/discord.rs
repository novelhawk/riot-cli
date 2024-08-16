use crate::models::WebhookMessage;

pub async fn send_webhook(url: &str, message: &WebhookMessage) {
    let client = reqwest::Client::new();
    client.post(url).json(&message).send().await.unwrap();
}
