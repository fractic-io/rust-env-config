pub mod errors;

use std::collections::HashMap;
use std::env;

use errors::{IncorrectConfigError, MissingEnvVariableError};
use fractic_generic_server_error::GenericServerError;

// Environment configuration.
// --------------------------------------------------

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum EnvVariable {
    CognitoRegion,
    CognitoUserPoolId,
    DynamoRegion,
    DynamoJourneyDB,
    PollyRegion,
}
impl EnvVariable {
    fn as_str(&self) -> &'static str {
        match *self {
            EnvVariable::CognitoRegion => "COGNITO_REGION",
            EnvVariable::CognitoUserPoolId => "COGNITO_USER_POOL_ID",
            EnvVariable::DynamoRegion => "DYNAMO_REGION",
            EnvVariable::DynamoJourneyDB => "DYNAMO_JOURNEYDB",
            EnvVariable::PollyRegion => "POLLY_REGION",
        }
    }
}

#[derive(Default)]
pub struct EnvVariables(HashMap<EnvVariable, String>);
impl EnvVariables {
    pub fn get(&self, key: &EnvVariable) -> Result<&String, GenericServerError> {
        let dbg_cxt: &'static str = "EnvVariables.get";
        self.0.get(key).ok_or(IncorrectConfigError::with_debug(
            dbg_cxt,
            "ENV variable missing",
            key.as_str().into(),
        ))
    }
}
impl<T> From<T> for EnvVariables
where
    T: Into<HashMap<EnvVariable, String>>,
{
    fn from(map: T) -> Self {
        EnvVariables(map.into())
    }
}

// Loads the environment variables provided, and throws an error if any are missing.
pub fn load_config(fields: Vec<EnvVariable>) -> Result<EnvVariables, GenericServerError> {
    let dbg_cxt: &'static str = "load_config";
    let mut map = HashMap::new();

    for field in fields {
        let value = env::var(field.as_str())
            .map_err(|_| MissingEnvVariableError::with_debug(dbg_cxt, "", field.as_str().into()))?;
        map.insert(field, value);
    }

    Ok(EnvVariables(map))
}

// Tests.
// --------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use once_cell::sync::Lazy;
    use std::env;
    use std::sync::Mutex;

    // Each test involving environment variables should be locked with ENV_LOCK.
    static ENV_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[test]
    fn test_env_variable_as_str() {
        // Just test a couple.
        assert_eq!(EnvVariable::CognitoRegion.as_str(), "COGNITO_REGION");
        assert_eq!(EnvVariable::DynamoRegion.as_str(), "DYNAMO_REGION");
        assert_eq!(EnvVariable::PollyRegion.as_str(), "POLLY_REGION");
    }

    #[test]
    fn test_load_config_partial_valid() {
        let _guard = ENV_LOCK.lock().unwrap();
        env::set_var("COGNITO_REGION", "us-west-2");
        env::remove_var("COGNITO_USER_POOL_ID");
        env::remove_var("DYNAMO_REGION");
        env::remove_var("POLLY_REGION");

        let expected_config: HashMap<EnvVariable, String> =
            [(EnvVariable::CognitoRegion, String::from("us-west-2"))].into();
        let EnvVariables(config) = load_config(vec![EnvVariable::CognitoRegion]).unwrap();
        assert_eq!(config, expected_config);
    }

    #[test]
    fn test_load_config_partial_invalid() {
        let _guard = ENV_LOCK.lock().unwrap();
        env::set_var("COGNITO_REGION", "us-west-2");
        env::remove_var("COGNITO_USER_POOL_ID");
        env::remove_var("DYNAMO_REGION");
        env::remove_var("POLLY_REGION");

        let config = load_config(vec![
            EnvVariable::CognitoRegion,
            EnvVariable::CognitoUserPoolId,
        ]);
        assert!(config.is_err());
    }

    #[test]
    fn test_load_config_var_not_set() {
        let _guard = ENV_LOCK.lock().unwrap();
        env::remove_var("COGNITO_REGION");

        let config = load_config(vec![EnvVariable::CognitoRegion]);
        assert!(config.is_err());
    }

    #[test]
    fn test_load_config_empty() {
        let fields: Vec<EnvVariable> = Vec::new();
        let EnvVariables(config) = load_config(fields).unwrap();
        assert_eq!(config, HashMap::new());
    }
}
