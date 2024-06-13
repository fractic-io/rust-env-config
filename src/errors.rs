use std::error::Error as StdError;
use std::fmt;

use fractic_generic_server_error::{
    define_internal_error_type, GenericServerError, GenericServerErrorTrait,
};

define_internal_error_type!(IncorrectConfigError, "Incorrect configuration.");
define_internal_error_type!(MissingEnvVariableError, "Missing environment variable.");
