use aws_config::BehaviorVersion;
use aws_sdk_secretsmanager::{config::Region, Client};
use fractic_server_error::common::CriticalError;
use fractic_server_error::GenericServerError;
use std::collections::HashMap;
use std::marker::PhantomData;

use crate::{define_env_config, EnvConfigEnum, EnvVariables, SECRETS_ID, SECRETS_REGION};

use super::errors::{
    FailedToFetchSecretsJson, InvalidSecretsConfig, MissingSecretKey, SecretsInvalidJson,
};

define_env_config!(
    SecretsEnvConfig,
    SecretsRegion => SECRETS_REGION,
    SecretsId => SECRETS_ID,
);

// Secrets configuration.
// --------------------------------------------------

// Similar set-up to EnvConfigEnum:
//
// define_secret_key!(OPENAI_KEY);
//
// define_secrets_config!(
//     SecretsConfig,
//     OpenAIKey => OPENAI_KEY,
// );
pub trait SecretsConfigEnum:
    std::fmt::Debug + PartialEq + Eq + core::hash::Hash + Clone + Send + Sync
{
    fn as_str(&self) -> &'static str;
    fn value_list() -> Vec<Self>;
}

// Similar to EnvVariables, fetch all secret values by running:
//
// let secrets: SecretValues<SecretsConfig> = load_secrets::<SecretsConfig>()?;
//
// The SecretValues object owns the data.
//
// This object is now guaranteed to have all the secret values for all keys in
// the SecretsConfig.
#[derive(Debug, Clone)]
pub struct SecretValues<T: SecretsConfigEnum>(HashMap<&'static str, String>, PhantomData<T>);
impl<T: SecretsConfigEnum> SecretValues<T> {
    pub fn get(&self, key: &T) -> Result<&String, GenericServerError> {
        self.get_raw(key.as_str())
    }
    fn get_raw(&self, key: &str) -> Result<&String, GenericServerError> {
        let dbg_cxt: &'static str = "SecretValues.get_raw";
        self.0.get(key).ok_or(CriticalError::with_debug(
            dbg_cxt,
            "Should be guaranteed any secret key SecretsConfig::key is present
            in SecretValues<SecretsConfig>, but it wasn't.",
            key.into(),
        ))
    }
}
pub async fn load_secrets<T: SecretsConfigEnum>(
    env: EnvVariables<SecretsEnvConfig>,
) -> Result<SecretValues<T>, GenericServerError> {
    let dbg_ctx: &'static str = "load_secrets";
    let region_str = env.get(&SecretsEnvConfig::SecretsRegion)?;
    let region = Region::new(region_str.clone());
    let shared_config = aws_config::defaults(BehaviorVersion::v2024_03_28())
        .region(region)
        .load()
        .await;
    let client = Client::new(&shared_config);

    // Fetch secrets JSON.
    let secrets_id = env.get(&SecretsEnvConfig::SecretsId)?;
    let secrets_output = client
        .get_secret_value()
        .secret_id(secrets_id)
        .send()
        .await
        .map_err(|e| {
            FailedToFetchSecretsJson::with_debug(
                dbg_ctx,
                "",
                format!(
                    "SecretsId: {}; Region: {}; Error: {:?};",
                    secrets_id, region_str, e
                ),
            )
        })?;
    let secrets_string = secrets_output
        .secret_string()
        .ok_or(CriticalError::with_debug(
            dbg_ctx,
            "could not parse secret value",
            format!("SecretsId: {}; Region: {};", secrets_id, region_str),
        ))?;
    let secrets_json =
        serde_json::from_str::<HashMap<String, String>>(secrets_string).map_err(|e| {
            SecretsInvalidJson::with_debug(
                dbg_ctx,
                "",
                format!(
                    "SecretsId: {}; Region: {}; Error: {};",
                    secrets_id,
                    region_str,
                    e.to_string()
                ),
            )
        })?;

    // Fetch required keys from JSON.
    let mut map = HashMap::new();
    for field in T::value_list() {
        let secret_value = secrets_json
            .get(field.as_str())
            .ok_or(MissingSecretKey::new(dbg_ctx, field.as_str()))?
            .as_str()
            .into();
        map.insert(field.as_str(), secret_value);
    }
    Ok(SecretValues(map.into(), PhantomData))
}

// For tests, let a SecretValues structure be easily made from a HashMap.
//
// let config: SecretValues<SecretsConfig> = collection! {
//     "OPENAI_KEY" => "abc123",
// };
impl<U, T: SecretsConfigEnum> From<U> for SecretValues<T>
where
    U: Into<HashMap<&'static str, String>>,
{
    fn from(map: U) -> Self {
        SecretValues(map.into(), PhantomData)
    }
}

// Like EnvVariables, a SecretValues object can be cloned into a smaller
// SecretValues as long as the child is a proper subset of the parent.
impl<ParentConfig: SecretsConfigEnum> SecretValues<ParentConfig> {
    pub fn clone_into<ChildConfig: SecretsConfigEnum>(
        &self,
    ) -> Result<SecretValues<ChildConfig>, GenericServerError> {
        let dbg_cxt: &'static str = "SecretValues::clone_into";
        let mut map = HashMap::new();
        for value in ChildConfig::value_list() {
            let key_as_str = value.as_str();
            let secret_value = self.get_raw(key_as_str).map_err(|_critical_error| {
                // Usually get_raw would return a critical error because the key
                // should always exist. However, when building a window, it
                // could be missing if the window config is not a proper subset
                // of the parent config. In this case, just let the developer
                // know the the parent SecretsConfig needs to be updated by
                // returning an InvalidSecretsConfig error.
                InvalidSecretsConfig::with_debug(dbg_cxt, "Secret key missing", key_as_str.into())
            })?;
            map.insert(key_as_str, secret_value.clone());
        }
        Ok(SecretValues(map, PhantomData))
    }
}

// Tests.
// --------------------------------------------------

// TODO
