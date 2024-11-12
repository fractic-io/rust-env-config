use fractic_server_error::define_internal_error;

define_internal_error!(
    InvalidEnvCloneInto,
    "Invalid clone_into(...). Parent config missing key '{missing_var}'.",
    { missing_var: &str }
);
define_internal_error!(
    MissingEnvVariableError,
    "Missing environment variable '{missing_var}'.",
    { missing_var: &str }
);
