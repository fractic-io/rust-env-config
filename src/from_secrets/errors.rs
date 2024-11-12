use fractic_server_error::define_internal_error;

define_internal_error!(
    FailedToFetchSecretsJson,
    "Failed to fetch secret '{secret_id}' (region '{region}') from Amazon Secrets Manager.",
    { secret_id: &str, region: &str }
);
define_internal_error!(
    MissingSecretKey,
    "Secret '{secret_id}' (region '{region}') missing key '{missing_key}'.",
    { secret_id: &str, region: &str, missing_key: &str }
);
define_internal_error!(
    SecretsInvalidJson,
    "Secret '{secret_id}' (region '{region}')'s value is not valid JSON.",
    { secret_id: &str, region: &str }
);
define_internal_error!(
    InvalidSecretsCloneInto,
    "Invalid clone_into(...). Parent config missing secret '{missing_secret}'.",
    { missing_secret: &str }
);
