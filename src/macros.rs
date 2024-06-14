#[macro_export]
macro_rules! define_env_variable {
    ($T:ident) => {
        pub static $T: &str = stringify!($T);
    };
}

#[macro_export]
macro_rules! define_env_config {
    ($T:ident, $($k:ident => $v:ident),* $(,)?) => {
        #[derive(Debug, PartialEq, Eq, Hash, Clone)]
        pub enum $T {
            $($k),*
        }

        impl EnvConfigEnum for $T {
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
