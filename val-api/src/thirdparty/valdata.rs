use crate::models::{SkinDetails, UnofficalApiResponse, ValorantVersionResponse};

pub async fn get_valorant_version() -> ValorantVersionResponse {
    const ENDPOINT: &str = "https://valorant-api.com/v1/version";
    let client = reqwest::Client::new();

    let res = client.get(ENDPOINT).send().await.unwrap();
    let res = res
        .json::<UnofficalApiResponse<ValorantVersionResponse>>()
        .await
        .unwrap();

    return res.data;
}

pub async fn get_client_version() -> String {
    let version = get_valorant_version().await;
    format!(
        "{}-shipping-{}-{}",
        version.branch,
        version.build_version,
        version.version.split('.').nth(3).unwrap()
    )
}

pub async fn get_weapon_skins() -> Vec<SkinDetails> {
    const ENDPOINT: &str = "https://valorant-api.com/v1/weapons/skins?language=en-US";
    let client = reqwest::Client::new();

    let res = client.get(ENDPOINT).send().await.unwrap();
    let res = res
        .json::<UnofficalApiResponse<Vec<SkinDetails>>>()
        .await
        .unwrap();

    res.data
}
