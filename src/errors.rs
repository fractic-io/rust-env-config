use fractic_server_error::{define_internal_error, ServerError, ServerErrorTrait};

// Env variables.
// --------------------------------------------------

define_internal_error!(
    InvalidEnvConfig,
    "Env variables needed by the window config are not present in the parent
    EnvConfig. Please update the parent config to include the required
    variable."
);
define_internal_error!(MissingEnvVariableError, "Missing environment variable.");

// Secret variables.
// --------------------------------------------------

define_internal_error!(
    FailedToFetchSecretsJson,
    "Failed to fetch secrets from Amazon Secrets Manager."
);
define_internal_error!(MissingSecretKey, "Missing key in secrets.");
define_internal_error!(SecretsInvalidJson, "Secrets value is not valid JSON.");
define_internal_error!(
    InvalidSecretsConfig,
    "Secret keys needed by the window config are not present in the parent
    SecretsConfig. Please update the parent config to include the required key."
);
