use std::collections::HashMap;
use std::io::Read;
use anyhow::{anyhow, Context};
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use libaes::Cipher;
use rand::Rng;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use crate::fq_api::register_key;

pub const REG_KEY: &'static str = "ac25c67ddd8f38c1b37a2348828e222e";

pub struct FqCrypto {
    cipher: Cipher,
}

impl FqCrypto {
    pub fn new(key: &str) -> anyhow::Result<Self> {
        let key = hex::decode(key)?;
        if key.len() != 16 {
            return Err(anyhow!("key length mismatch!\nkey: {:?}", key));
        }
        let cipher = Cipher::new_128(key[..16].try_into()?);
        Ok(
            Self {
                cipher
            }
        )
    }


    pub fn encrypt(self: &Self, data: &[u8], iv: &[u8]) -> anyhow::Result<Vec<u8>> {
        let res = self.cipher.cbc_encrypt(&iv, data);

        if res.is_empty() {
            Err(anyhow!("encrypt failed"))
        } else {
            Ok(res)
        }
    }


    pub fn decrypt(self: &Self, data: &str) -> anyhow::Result<Vec<u8>> {
        let decoded_data = BASE64_STANDARD.decode(data).context("failed to decode data")?;

        let iv = &decoded_data[..16];
        let encrypted_data = &decoded_data[16..];
        let res = self.cipher.cbc_decrypt(iv, encrypted_data);

        if res.is_empty() {
            Err(anyhow!("decrypt failed"))
        } else {
            Ok(res)
        }
    }


    pub fn new_register_key_content(self: &Self, server_device_id: &str, str_val: &str) -> anyhow::Result<String> {
        let (server_device_id, str_val) = match (server_device_id.parse::<i64>(), str_val.parse::<i64>()) {
            (Ok(lhs), Ok(rhs)) => (lhs, rhs),
            _ => return Err(anyhow!("parse failed\nserver_device_id: {}\nstr_val:{}", server_device_id, str_val))
        };

        let combined_bytes = [server_device_id.to_ne_bytes(), str_val.to_ne_bytes()].concat();
        let mut iv = [0u8; 16];  // Initialize an array with 16 bytes
        rand::thread_rng().fill(&mut iv);  // Fill the array with random bytes

        let enc_data = self.encrypt(combined_bytes.as_slice(), &iv)?;
        let combined_bytes = [&iv, enc_data.as_slice()].concat();
        Ok(BASE64_STANDARD.encode(combined_bytes))
    }
}


pub struct FqVariable {
    pub install_id: String,
    pub server_device_id: String,
    pub aid: String,
    pub update_version_code: String,
}


#[derive(Serialize, Deserialize)]
pub struct FqRegisterKeyPayload {
    #[serde(alias = "key")]
    pub content: String,
    pub keyver: i64,
}
impl FqRegisterKeyPayload {
    pub fn new(var: &FqVariable) -> anyhow::Result<Self> {
        let crypto = FqCrypto::new(REG_KEY)?;
        let content = crypto.new_register_key_content(var.server_device_id.as_str(), "0")?;
        Ok(Self { content, keyver: 1 })
    }

    pub fn get_key(&self) -> anyhow::Result<String> {
        let crypto = FqCrypto::new(REG_KEY)?;
        let byte_key = crypto.decrypt(self.content.as_str())?;
        Ok(hex::encode(byte_key))
    }
}
#[derive(Serialize, Deserialize)]
pub struct FqRegisterKeyResponse {
    pub code: i64,
    pub message: String,
    pub data: FqRegisterKeyPayload,
}

#[derive(Serialize, Deserialize)]
pub struct FqIBatchFullResponse {
    pub code: i64,
    pub message: String,
    pub data: HashMap<String, ItemContent>,
}

#[derive(Serialize, Deserialize)]
pub struct ItemContent {
    pub code: i64,
    pub title: String,
    pub content: String,
    pub novel_data: Value,
    pub text_type: i64,
    pub crypt_status: i64,
    pub compress_status: i64,
    pub key_version: i64,
    pub paragraphs_num: i64,
    // pub author_speak: String,
}

impl FqIBatchFullResponse {
    pub async fn get_decrypt_contents(&self, client: &Client, var: &FqVariable) -> anyhow::Result<Vec<(String, String)>> {
        use flate2::read::GzDecoder;

        let register_key = register_key(client, var).await?;
        let key = register_key.data.get_key()?;

        let mut res = Vec::new();

        for (item_id, content) in &self.data {
            let crypto = FqCrypto::new(key.as_str())?;
            let byte_content = crypto.decrypt(content.content.as_str())?;

            let mut d = GzDecoder::new(byte_content.as_slice());
            let mut s = String::new();
            d.read_to_string(&mut s)?;
            res.push((item_id.clone(), s));
        }
        Ok(res)
    }
}