use aws_config::meta::region::RegionProviderChain;
use aws_sdk_secretsmanager::Client;
use serde_json::Value;

pub async fn get_secret_env_values_from_keys(
    region: &'static str,
    secret_name: &str,
    env_keys: Vec<&str>,
) -> anyhow::Result<Vec<String>> {
    let secrets = get_secret_env_value(region, secret_name).await?;

    let values = env_keys
        .iter()
        .map(|key| {
            secrets[key]
                .as_str()
                .map(|value| value.to_string())
                .ok_or_else(|| anyhow::bail!(format!("missing key: {}", key)))
                .unwrap_or_else(|_: anyhow::Result<()>| unreachable!())
        })
        .collect::<Vec<String>>();

    Ok(values)
}

pub async fn get_secret_env_value(
    region: &'static str,
    secret_name: &str,
) -> anyhow::Result<Value> {
    let region_provider = RegionProviderChain::default_provider().or_else(region);
    let config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&config);

    let response = client
        .get_secret_value()
        .secret_id(secret_name)
        .send()
        .await?;

    let value: &str = response
        .secret_string()
        .ok_or(anyhow::anyhow!("fail to get secret string from id"))?;

    let secret_info: Value = serde_json::from_str(value)?;

    Ok(secret_info)
}
