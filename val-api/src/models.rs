use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use val_login_webview::tokens::Tokens;

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthClientRequest {
    pub client_id: String,
    pub nonce: String,
    pub redirect_uri: String,
    pub response_type: String,
    pub scope: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginRequest {
    #[serde(rename = "type")]
    pub action_type: String,
    pub username: String,
    pub password: String,
    pub remember: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginResponseParameters {
    pub uri: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginResponseData {
    pub parameters: LoginResponseParameters,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginResponse {
    #[serde(rename = "type")]
    pub response_type: String,
    pub response: LoginResponseData,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthResult {
    pub tokens: Tokens,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub cookies: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EntitlementsTokenResponse {
    pub entitlements_token: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UnofficalApiResponse<T> {
    pub status: u16,
    pub data: T,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ValorantVersionResponse {
    pub manifest_id: String,         //"08FD21AD41B7E3D3",
    pub branch: String,              //"release-05.05",
    pub version: String,             //"05.05.00.759728",
    pub build_version: String,       //"7",
    pub engine_version: String,      //"4.26.2.0",
    pub riot_client_version: String, //"release-05.05-shipping-7-759728",
    pub build_date: String,          //"2022-09-06T00:00:00Z"
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PasswordDataResponse {
    cng_at: u64,
    reset: bool,
    must_reset: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AcctDataResponse {
    #[serde(rename = "type")]
    pub acct_type: u64,
    pub state: Option<String>,
    pub adm: bool,
    pub game_name: String,
    pub tag_line: String,
    pub created_at: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserInfoResponse {
    country: String,
    pub sub: String,
    email_verified: bool,
    player_plocale: Option<String>,
    pw: PasswordDataResponse,
    country_at: Option<u64>,
    phone_number_verified: bool,
    account_verified: bool,
    // ppid: Option<Idk>,
    player_locale: String,
    pub acct: AcctDataResponse,
    age: Option<u64>,
    jti: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegionRequest {
    pub id_token: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Affinities {
    pub pbe: String,
    pub live: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegionResponse {
    pub token: String,
    pub affinities: Affinities,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkinDetails {
    pub uuid: String,
    pub display_name: Option<String>,
    pub display_icon: Option<String>,
    pub levels: Vec<SkinLevel>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkinLevel {
    pub uuid: String,
    pub display_name: Option<String>,
    pub display_icon: Option<String>,
    pub streamed_video: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct SkinPanelLayout {
    pub single_item_offers: Vec<String>,
    pub single_item_offers_remaining_duration_in_seconds: i64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct StoreFrontResponse {
    pub skins_panel_layout: SkinPanelLayout,
    pub bonus_store: Option<BonusStore>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct BonusStore {
    pub bonus_store_offers: Vec<BonusStoreOffer>,
    pub bonus_store_remaining_duration_in_seconds: i64,
    pub bonus_store_seconds_since_it_started: i64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct BonusStoreOffer {
    #[serde(rename = "BonusOfferID")]
    pub bonus_offer_id: String,
    pub offer: Offer,
    pub discount_percent: i64,
    pub discount_costs: PriceDetail,
    pub is_seen: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Offer {
    #[serde(rename = "OfferID")]
    pub offer_id: String,
    pub is_direct_purchase: bool,
    pub start_date: DateTime<Utc>,
    pub cost: PriceDetail,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PriceDetail {
    #[serde(rename = "85ad13f7-3d1b-5128-9eb2-7cd8ee0b5741")]
    pub valorant_points: u64,
    // #[serde(skip_serializing)]
    // pub radianite: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EmbedFooter {
    pub text: Option<String>,
    pub icon_url: Option<String>,
    pub proxy_icon_url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EmbedImage {
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MessageEmbed {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<EmbedImage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub footer: Option<EmbedFooter>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WebhookMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embeds: Option<Vec<MessageEmbed>>,
}
