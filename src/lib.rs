use aws_config::meta::region::RegionProviderChain;
use aws_sdk_cognitoidentityprovider as cognitoidentityprovider;
use aws_sdk_secretsmanager::{operation::get_secret_value::GetSecretValueOutput, Client};
use reqwest::header::{HeaderValue, AUTHORIZATION};
use serde_json::Value;

#[allow(dead_code)]
async fn send_request_with_token(token: &str) -> anyhow::Result<reqwest::Response> {
    // トークンをAuthorizationヘッダーに含める
    let client = reqwest::Client::new();
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", token))?,
    );

    // APIリクエストを送信
    let response = client
        .get("http://localhost:9000/")
        .headers(headers)
        .send()
        .await?;

    Ok(response)
}

// secret_keyはクライアントシークレットのこと
#[allow(dead_code)]
fn get_secret_hash(username: &str, secret_key: &str, client_id: &str) -> anyhow::Result<String> {
    use base64::Engine as _;
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    let mut mac = Hmac::<Sha256>::new_from_slice(secret_key.as_bytes())?;
    mac.update(username.as_bytes());
    mac.update(client_id.as_bytes());

    let result = mac.finalize().into_bytes();
    let secret_hash = base64::engine::general_purpose::STANDARD.encode(result);

    Ok(secret_hash)
}

#[allow(dead_code)]
async fn get_id_token(
    username: &str,
    password: &str,
    secret_hash: &str,
    client_id: &str,
    user_pool_id: &str,
) -> anyhow::Result<String> {
    let config = aws_config::load_from_env().await;

    let client = cognitoidentityprovider::Client::new(&config);

    let response_builder = client
        .admin_initiate_auth()
        .auth_flow(cognitoidentityprovider::types::AuthFlowType::AdminUserPasswordAuth)
        .auth_parameters("USERNAME".to_string(), username.to_string())
        .auth_parameters("PASSWORD".to_string(), password.to_string())
        .auth_parameters("SECRET_HASH".to_string(), secret_hash.to_string())
        .client_id(client_id.to_string())
        .user_pool_id(user_pool_id.to_string());

    let response = response_builder.send().await;
    let response = response?;

    let id_token = response
        .authentication_result()
        .ok_or(anyhow::anyhow!("missing authentication result"))?
        .id_token()
        .ok_or(anyhow::anyhow!("missing id token"))?;

    Ok(id_token.to_string())
}

pub async fn get_secret_value(region: &'static str, secret_name: &str) -> anyhow::Result<Value> {
    let region_provider = RegionProviderChain::default_provider().or_else(region);

    let config = aws_config::from_env().region(region_provider).load().await;

    let client = Client::new(&config);

    let get_secret_value = client.get_secret_value();

    let secret_id = get_secret_value.secret_id(secret_name);

    let sent = secret_id.send().await;

    let resp = sent.unwrap_or(GetSecretValueOutput::builder().build());

    let value: &str = resp
        .secret_string()
        .ok_or(anyhow::anyhow!("fail to get secret string from id"))?;

    let secret_info: Value = serde_json::from_str(value)?;

    Ok(secret_info)
}
