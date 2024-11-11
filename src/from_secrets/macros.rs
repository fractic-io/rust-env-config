#[macro_export]
macro_rules! define_secret_key {
    ($T:ident) => {
        pub static $T: &str = stringify!($T);
    };
}

#[macro_export]
macro_rules! define_secrets_config {
    ($T:ident, $($k:ident => $v:ident),* $(,)?) => {
        #[derive(Debug, PartialEq, Eq, Hash, Clone)]
        pub enum $T {
            $($k),*
        }

        impl SecretsConfigEnum for $T {
            fn as_str(&self) -> &'static str {
                match *self {
                    $($T::$k => $v),*
                }
            }

            fn value_list() -> Vec<Self> {
                [$($T::$k),*].to_vec()
            }
        }
    };
}

#[cfg(test)]
mod macro_tests {
    use crate::{define_secret_key, define_secrets_config, SecretValues, SecretsConfigEnum};
    use std::collections::HashMap;

    #[test]
    fn test_define_env_variable() {
        define_secret_key!(TEST_SECRET_VAR);
        assert_eq!(TEST_SECRET_VAR, "TEST_SECRET_VAR");
    }

    #[test]
    fn test_define_env_config() {
        define_secret_key!(TEST_SECRET_VAR_1);
        define_secret_key!(TEST_SECRET_VAR_2);

        define_secrets_config!(
            TestConfig,
            TestVar1 => TEST_SECRET_VAR_1,
            TestVar2 => TEST_SECRET_VAR_2,
        );

        assert_eq!(TestConfig::TestVar1.as_str(), "TEST_SECRET_VAR_1");
        assert_eq!(TestConfig::TestVar2.as_str(), "TEST_SECRET_VAR_2");

        let expected_list = vec![TestConfig::TestVar1, TestConfig::TestVar2];
        assert_eq!(TestConfig::value_list(), expected_list);
    }

    #[test]
    fn test_env_variables_with_macros() {
        define_secret_key!(TEST_SECRET_VAR_1);
        define_secret_key!(TEST_SECRET_VAR_2);

        define_secrets_config!(
            TestConfig,
            TestVar1 => TEST_SECRET_VAR_1,
            TestVar2 => TEST_SECRET_VAR_2,
        );

        let mut env_map = HashMap::new();
        env_map.insert(TEST_SECRET_VAR_1, String::from("value1"));
        env_map.insert(TEST_SECRET_VAR_2, String::from("value2"));

        let env_variables: SecretValues<TestConfig> = SecretValues::from(env_map);

        assert_eq!(env_variables.get(&TestConfig::TestVar1).unwrap(), "value1");
        assert_eq!(env_variables.get(&TestConfig::TestVar2).unwrap(), "value2");
    }

    #[test]
    fn test_define_env_config_empty() {
        define_secrets_config!(EmptyConfig,);
        let expected_list: Vec<EmptyConfig> = vec![];
        assert_eq!(EmptyConfig::value_list(), expected_list);
    }

    #[test]
    fn test_define_env_config_single() {
        define_secret_key!(TEST_SECRET_VAR_SINGLE);

        define_secrets_config!(
            SingleConfig,
            SingleVar => TEST_SECRET_VAR_SINGLE,
        );

        assert_eq!(SingleConfig::SingleVar.as_str(), "TEST_SECRET_VAR_SINGLE");

        let expected_list = vec![SingleConfig::SingleVar];
        assert_eq!(SingleConfig::value_list(), expected_list);
    }
}
