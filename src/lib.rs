pub mod errors;
pub mod macros;

use std::collections::HashMap;
use std::env;
use std::marker::PhantomData;

use errors::{InvalidEnvConfig, MissingEnvVariableError};
use fractic_generic_server_error::{common::CriticalError, GenericServerError};

// Environment configuration.
// --------------------------------------------------

// For any given execution environment (function, library, etc.), they should
// define an enum implementing this trait, which contains all the environment
// variables needed for that code to run.
//
// The relevent config can be easily defined using the macros from macros.rs:
//
// define_env_variable!(COGNITO_REGION);
// define_env_variable!(COGNITO_USER_POOL_ID);
//
// define_env_config!(
//     EnvConfig,
//     CognitoRegion => COGNITO_REGION,
//     DynamoRegion => DYNAMO_REGION,
// );
//
// Now the EnvConfig object can be used to fetch and manage the environment
// variable values in a way that's largely type-checked by the compiler.
pub trait EnvConfigEnum:
    std::fmt::Debug + PartialEq + Eq + core::hash::Hash + Clone + Send + Sync
{
    fn as_str(&self) -> &'static str;
    fn value_list() -> Vec<Self>;
}

// To initialize a given environment, call load_env::<EnvConfig>() to fetch all
// the environment variable values for that config and store them in a
// EnvVariables object (which is essentiall just a map).
//
// let config: EnvVariables<EnvConfig> = load_env::<EnvConfig>()?;
//
// The EnvVariables object owns the data.
//
// This object is now guaranteed to have all the values for each enum value of
// EnvConfig, and it is compiler-ensured that you don't accidentally try to
// access any variables that were not specified in the config.
#[derive(Debug, Clone)]
pub struct EnvVariables<T: EnvConfigEnum>(HashMap<&'static str, String>, PhantomData<T>);
impl<T: EnvConfigEnum> EnvVariables<T> {
    pub fn get(&self, key: &T) -> Result<&String, GenericServerError> {
        self.get_raw(key.as_str())
    }
    fn get_raw(&self, key: &str) -> Result<&String, GenericServerError> {
        let dbg_cxt: &'static str = "EnvVariables.get_raw";
        self.0.get(key).ok_or(CriticalError::with_debug(
            dbg_cxt,
            "Should be guaranteed any ENV variable EnvConfig::key is present in
            EnvVariables<EnvConfig>, but it wasn't.",
            key.into(),
        ))
    }
}
pub fn load_env<T: EnvConfigEnum>() -> Result<EnvVariables<T>, GenericServerError> {
    let dbg_cxt: &'static str = "load_config";
    let mut map = HashMap::new();

    for field in T::value_list() {
        let value = env::var(field.as_str())
            .map_err(|_| MissingEnvVariableError::with_debug(dbg_cxt, "", field.as_str().into()))?;
        map.insert(field.as_str(), value);
    }

    Ok(EnvVariables(map, PhantomData))
}

// For tests, let an EnvVariables structure be easily made from a HashMap.
//
// let config: EnvVariables<EnvConfig> = collection! {
//     "COGNITO_REGION" => "us-west-2",
//     "COGNITO_USER_POOL_ID" => "us-west-2",
// };
impl<U, T: EnvConfigEnum> From<U> for EnvVariables<T>
where
    U: Into<HashMap<&'static str, String>>,
{
    fn from(map: U) -> Self {
        EnvVariables(map.into(), PhantomData)
    }
}

// An EnvVariables object can be cloned into a smaller EnvVariables as long as
// the child is a proper subset of the parent.
impl<ParentConfig: EnvConfigEnum> EnvVariables<ParentConfig> {
    pub fn clone_into<ChildConfig: EnvConfigEnum>(
        &self,
    ) -> Result<EnvVariables<ChildConfig>, GenericServerError> {
        let dbg_cxt: &'static str = "EnvVariables::clone_into";
        let mut map = HashMap::new();
        for value in ChildConfig::value_list() {
            let key_as_str = value.as_str();
            let env_value = self.get_raw(key_as_str).map_err(|_critical_error| {
                // Usually get_raw would return a critical error because the key
                // should always exist. However, when building a window, it
                // could be missing if the window config is not a proper subset
                // of the parent config. In this case, just let the developer
                // know the the parent EnvConfig needs to be updated by
                // returning an InvalidEnvConfig error.
                InvalidEnvConfig::with_debug(dbg_cxt, "ENV variable missing", key_as_str.into())
            })?;
            map.insert(key_as_str, env_value.clone());
        }
        Ok(EnvVariables(map, PhantomData))
    }
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

    define_env_variable!(COGNITO_REGION);
    define_env_variable!(COGNITO_USER_POOL_ID);
    define_env_variable!(DYNAMO_REGION);
    define_env_variable!(POLLY_REGION);

    define_env_config!(
        AllVariablesConfig,
        CognitoRegion => COGNITO_REGION,
        CognitoUserPoolId => COGNITO_USER_POOL_ID,
        DynamoRegion => DYNAMO_REGION,
        PollyRegion => POLLY_REGION,
    );

    define_env_config!(
        CognitoRegionOnlyConfig,
        CognitoRegion => COGNITO_REGION,
    );

    define_env_config!(EmptyConfig,);

    #[test]
    fn test_env_variable_as_str() {
        // Just test a couple.
        assert_eq!(AllVariablesConfig::CognitoRegion.as_str(), "COGNITO_REGION");
        assert_eq!(AllVariablesConfig::DynamoRegion.as_str(), "DYNAMO_REGION");
        assert_eq!(AllVariablesConfig::PollyRegion.as_str(), "POLLY_REGION");
    }

    #[test]
    fn test_load_config_partial_valid() {
        let _guard = ENV_LOCK.lock().unwrap();
        env::set_var("COGNITO_REGION", "us-west-2");
        env::remove_var("COGNITO_USER_POOL_ID");
        env::remove_var("DYNAMO_REGION");
        env::remove_var("POLLY_REGION");

        let expected_config: HashMap<&'static str, String> =
            [(COGNITO_REGION, String::from("us-west-2"))].into();
        let EnvVariables(config, PhantomData) = load_env::<CognitoRegionOnlyConfig>().unwrap();
        assert_eq!(config, expected_config);
    }

    #[test]
    fn test_load_config_partial_invalid() {
        let _guard = ENV_LOCK.lock().unwrap();
        env::set_var("COGNITO_REGION", "us-west-2");
        env::remove_var("COGNITO_USER_POOL_ID");
        env::remove_var("DYNAMO_REGION");
        env::remove_var("POLLY_REGION");

        let config = load_env::<AllVariablesConfig>();
        assert!(config.is_err());
    }

    #[test]
    fn test_load_config_var_not_set() {
        let _guard = ENV_LOCK.lock().unwrap();
        env::remove_var("COGNITO_REGION");

        let config = load_env::<CognitoRegionOnlyConfig>();
        assert!(config.is_err());
    }

    #[test]
    fn test_load_config_empty() {
        let EnvVariables(config, PhantomData) = load_env::<EmptyConfig>().unwrap();
        assert_eq!(config, HashMap::new());
    }

    #[test]
    fn test_subset_valid() {
        let input_map: HashMap<&'static str, String> = [
            (COGNITO_REGION, String::from("us-west-2")),
            (COGNITO_USER_POOL_ID, String::from("pool-id")),
            (DYNAMO_REGION, String::from("us-west-2")),
            (POLLY_REGION, String::from("us-east-1")),
        ]
        .into();

        let env_variables: EnvVariables<AllVariablesConfig> = EnvVariables::from(input_map);
        let subset: EnvVariables<CognitoRegionOnlyConfig> = env_variables.clone_into().unwrap();

        assert_eq!(
            subset.get(&CognitoRegionOnlyConfig::CognitoRegion).unwrap(),
            "us-west-2"
        );
    }

    #[test]
    fn test_subset_invalid() {
        let input_map: HashMap<&'static str, String> = [
            (COGNITO_USER_POOL_ID, String::from("pool-id")),
            (DYNAMO_REGION, String::from("us-west-2")),
        ]
        .into();

        let env_variables: EnvVariables<AllVariablesConfig> = EnvVariables::from(input_map);
        let subset_result = env_variables.clone_into::<CognitoRegionOnlyConfig>();

        assert!(subset_result.is_err());
    }

    #[test]
    fn test_subset_empty() {
        let input_map: HashMap<&'static str, String> =
            [(COGNITO_REGION, String::from("us-west-2"))].into();

        let env_variables: EnvVariables<AllVariablesConfig> = EnvVariables::from(input_map);
        let subset: EnvVariables<EmptyConfig> = env_variables.clone_into().unwrap();

        assert_eq!(subset.0, HashMap::new());
    }

    #[test]
    fn test_env_variables_from_hashmap() {
        let input_map: HashMap<&'static str, String> = [
            (COGNITO_REGION, String::from("us-west-2")),
            (DYNAMO_REGION, String::from("us-west-2")),
        ]
        .into();

        let env_variables: EnvVariables<AllVariablesConfig> = EnvVariables::from(input_map);

        assert_eq!(
            env_variables
                .get(&AllVariablesConfig::CognitoRegion)
                .unwrap(),
            "us-west-2"
        );
        assert_eq!(
            env_variables
                .get(&AllVariablesConfig::DynamoRegion)
                .unwrap(),
            "us-west-2"
        );
    }

    #[test]
    fn test_env_variables_get_invalid_key() {
        let _guard = ENV_LOCK.lock().unwrap();
        let input_map: HashMap<&'static str, String> =
            [(COGNITO_REGION, String::from("us-west-2"))].into();
        let env_variables: EnvVariables<AllVariablesConfig> = EnvVariables::from(input_map);
        let result = env_variables.get(&AllVariablesConfig::PollyRegion);
        assert!(result.is_err());
    }
}
