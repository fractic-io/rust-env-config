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
pub trait EnvConfigEnum: std::fmt::Debug + PartialEq + Eq + core::hash::Hash + Clone {
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

// Libraries and functions can use a subset of the env variables by using a
// "window", which does not take ownership but only takes references to the
// required variables.
//
// let window: EnvVariablesWindow<EnvConfigWindow> = build_env_window(&config)?;
//
// Again, this now statically insures that all the variables specified in the
// window's config are available, and that the child code / library does not
// accidentally access any variables outside of that.
pub struct EnvVariablesWindow<'env_variables_life, T: EnvConfigEnum>(
    HashMap<&'static str, &'env_variables_life String>,
    PhantomData<T>,
);
impl<T: EnvConfigEnum> EnvVariablesWindow<'_, T> {
    pub fn get(&self, key: &T) -> Result<&String, GenericServerError> {
        self.get_raw(key.as_str())
    }
    fn get_raw(&self, key: &str) -> Result<&String, GenericServerError> {
        let dbg_cxt: &'static str = "EnvVariables.get_raw";
        Ok(*(self.0.get(key).ok_or(CriticalError::with_debug(
            dbg_cxt,
            "Should be guaranteed any ENV variable EnvConfig::key is present in
            EnvVariablesWindow<EnvConfig>, but it wasn't.",
            key.into(),
        ))?))
    }
}
pub fn build_env_window<
    'env_variables_life,
    ParentConfig: EnvConfigEnum,
    WindowConfig: EnvConfigEnum,
>(
    parent: &'env_variables_life EnvVariables<ParentConfig>,
) -> Result<EnvVariablesWindow<'env_variables_life, WindowConfig>, GenericServerError> {
    let dbg_cxt: &'static str = "build_env_window";
    let mut map = HashMap::new();
    for value in WindowConfig::value_list() {
        let key_as_str = value.as_str();
        map.insert(
            key_as_str,
            parent.get_raw(key_as_str).map_err(|_critical_error| {
                // Usually get_raw would return a critical error because the key
                // should always exist. However, when building a window, it
                // could be missing if the window config is not a proper subset
                // of the parent config. In this case, just let the developer
                // know the the parent EnvConfig needs to be updated by
                // returning an InvalidEnvConfig error.
                InvalidEnvConfig::with_debug(dbg_cxt, "ENV variable missing", key_as_str.into())
            })?,
        );
    }
    Ok(EnvVariablesWindow(map, PhantomData))
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
    define_env_variable!(DYNAMO_JOURNEYDB);
    define_env_variable!(POLLY_REGION);

    define_env_config!(
        TestAllVariablesConfig,
        CognitoRegion => COGNITO_REGION,
        CognitoUserPoolId => COGNITO_USER_POOL_ID,
        DynamoRegion => DYNAMO_REGION,
        DynamoJourneyDB => DYNAMO_JOURNEYDB,
        PollyRegion => POLLY_REGION,
    );

    define_env_config!(
        TestOneVariableConfig,
        CognitoRegion => COGNITO_REGION,
    );

    define_env_config!(EmptyConfig,);

    #[test]
    fn test_env_variable_as_str() {
        // Just test a couple.
        assert_eq!(
            TestAllVariablesConfig::CognitoRegion.as_str(),
            "COGNITO_REGION"
        );
        assert_eq!(
            TestAllVariablesConfig::DynamoRegion.as_str(),
            "DYNAMO_REGION"
        );
        assert_eq!(TestAllVariablesConfig::PollyRegion.as_str(), "POLLY_REGION");
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
        let EnvVariables(config, PhantomData) = load_env::<TestOneVariableConfig>().unwrap();
        assert_eq!(config, expected_config);
    }

    #[test]
    fn test_load_config_partial_invalid() {
        let _guard = ENV_LOCK.lock().unwrap();
        env::set_var("COGNITO_REGION", "us-west-2");
        env::remove_var("COGNITO_USER_POOL_ID");
        env::remove_var("DYNAMO_REGION");
        env::remove_var("POLLY_REGION");

        let config = load_env::<TestAllVariablesConfig>();
        assert!(config.is_err());
    }

    #[test]
    fn test_load_config_var_not_set() {
        let _guard = ENV_LOCK.lock().unwrap();
        env::remove_var("COGNITO_REGION");

        let config = load_env::<TestOneVariableConfig>();
        assert!(config.is_err());
    }

    #[test]
    fn test_load_config_empty() {
        let EnvVariables(config, PhantomData) = load_env::<EmptyConfig>().unwrap();
        assert_eq!(config, HashMap::new());
    }
}
