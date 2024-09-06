use reqwest::Client;
use crate::fq_struct::{FqCrypto, FqIBatchFullResponse, FqRegisterKeyPayload, FqRegisterKeyResponse, FqVariable};

pub async fn batch_full(client: &Client, var: &FqVariable, item_ids: &str, download: bool) -> anyhow::Result<FqIBatchFullResponse> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Cookie", format!("install_id={}", var.install_id).parse()?);

    let url = "https://api5-normal-sinfonlineb.fqnovel.com/reading/reader/batch_full/v";
    let params = [
        ("item_ids", item_ids),
        ("req_type", if download { "0" } else { "1" }),
        ("aid", var.aid.as_str()),
        // ("version_code", var.update_version_code),
        ("update_version_code", var.update_version_code.as_str())
    ];

    let url = reqwest::Url::parse_with_params(url, &params)?;
    let request = client.request(reqwest::Method::GET, url)
        .headers(headers);

    let response = request.send().await?;
    let body = response.json::<FqIBatchFullResponse>().await?;

    Ok(body)
}

pub async fn register_key(client: &Client, var: &FqVariable) -> anyhow::Result<FqRegisterKeyResponse> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Cookie", format!("install_id={}", var.install_id).parse()?);

    let url = "https://api5-normal-sinfonlineb.fqnovel.com/reading/crypt/registerkey";
    let params = [
        ("aid", var.aid.as_str()),
    ];

    let url = reqwest::Url::parse_with_params(url, &params)?;
    let payloads = FqRegisterKeyPayload::new(var)?;

    let request = client.request(reqwest::Method::POST, url)
        .headers(headers)
        .json(&payloads);

    let response = request.send().await?;
    let body = response.json::<FqRegisterKeyResponse>().await?;

    Ok(body)
}