use fractic_server_error::{
    define_internal_error_type, GenericServerError, GenericServerErrorTrait,
};

// Env variables.
// --------------------------------------------------

define_internal_error_type!(
    InvalidEnvConfig,
    "Env variables needed by the window config are not present in the parent
    EnvConfig. Please update the parent config to include the required
    variable."
);
define_internal_error_type!(MissingEnvVariableError, "Missing environment variable.");

// Secret variables.
// --------------------------------------------------

define_internal_error_type!(
    FailedToFetchSecretsJson,
    "Failed to fetch secrets from Amazon Secrets Manager."
);
define_internal_error_type!(MissingSecretKey, "Missing key in secrets.");
define_internal_error_type!(SecretsInvalidJson, "Secrets value is not valid JSON.");
define_internal_error_type!(
    InvalidSecretsConfig,
    "Secret keys needed by the window config are not present in the parent
    SecretsConfig. Please update the parent config to include the required key."
);
