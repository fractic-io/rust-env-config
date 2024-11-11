use fractic_server_error::{
    define_internal_error_type, GenericServerError, GenericServerErrorTrait,
};

define_internal_error_type!(
    InvalidEnvConfig,
    "Env variables needed by the window config are not present in the parent
    EnvConfig. Please update the parent config to include the required
    variable."
);
define_internal_error_type!(MissingEnvVariableError, "Missing environment variable.");
