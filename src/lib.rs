use aws_config::meta::region::RegionProviderChain;
use aws_sdk_secretsmanager::{operation::get_secret_value::GetSecretValueOutput, Client};
use serde_json::Value;

pub async fn get_secret_value(region: &'static str, secret_name: &str) -> anyhow::Result<Value> {
    let region_provider = RegionProviderChain::default_provider().or_else(region);
    tracing::info!("1/9 set default provider");

    let config = aws_config::from_env().region(region_provider).load().await;
    tracing::info!("2/9 loaded aws config from env region");

    let client = Client::new(&config);
    tracing::info!("3/9 set client to config");

    let get_secret_value = client.get_secret_value();
    tracing::info!("4/9 got secret value by client");

    let secret_id = get_secret_value.secret_id(secret_name);
    tracing::info!("5/9 set secret id from value");

    let sent = secret_id.send().await;
    tracing::info!("6/9 send secret id");

    let resp = sent.unwrap_or(GetSecretValueOutput::builder().build());
    tracing::info!("7/9 built response from sent id");

    let value: &str = resp
        .secret_string()
        .ok_or(anyhow::anyhow!("fail to get secret string from id"))?;
    tracing::info!("8/9 got secret string from response");

    let secret_info: Value = serde_json::from_str(value)?;
    tracing::info!("9/9 set secret info from value");

    Ok(secret_info)
}
