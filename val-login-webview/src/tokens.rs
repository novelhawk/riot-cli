use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Tokens {
    pub access_token: String,
    pub id_token: String,
    pub expires_in: u64,
}
