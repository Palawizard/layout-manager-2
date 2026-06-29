use serde::Serialize;
use thiserror::Error;

use crate::domain::ports::NativeError;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("invalid input: {0}")]
    Validation(String),
    #[error("the requested resource was not found")]
    NotFound,
    #[error("storage error: {0}")]
    Storage(String),
    #[error("an internal error occurred")]
    Internal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicError {
    pub code: &'static str,
    pub message: String,
    pub field: Option<String>,
    pub retryable: bool,
}

impl From<AppError> for PublicError {
    fn from(error: AppError) -> Self {
        match error {
            AppError::Validation(message) => Self {
                code: "validation_failed",
                message,
                field: None,
                retryable: false,
            },
            AppError::NotFound => Self {
                code: "not_found",
                message: "Élément introuvable.".to_owned(),
                field: None,
                retryable: false,
            },
            AppError::Storage(_) => Self {
                code: "storage_error",
                message: "Les données n’ont pas pu être enregistrées.".to_owned(),
                field: None,
                retryable: true,
            },
            AppError::Internal => Self {
                code: "internal_error",
                message: "Une erreur est survenue.".to_owned(),
                field: None,
                retryable: true,
            },
        }
    }
}

impl From<NativeError> for PublicError {
    fn from(error: NativeError) -> Self {
        let (code, message, retryable) = match error {
            NativeError::AccessDenied => (
                "access_denied",
                crate::domain::ports::ACCESS_DENIED_USER_MESSAGE.to_owned(),
                false,
            ),
            NativeError::InvalidHandle => {
                ("window_not_found", "Fenêtre introuvable.".to_owned(), true)
            }
            NativeError::OperationFailed(_) => (
                "native_operation_failed",
                "L’opération Windows n’a pas abouti.".to_owned(),
                true,
            ),
        };
        Self {
            code,
            message,
            field: None,
            retryable,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{AppError, PublicError};

    #[test]
    fn serializes_public_errors_for_the_frontend() {
        let error = PublicError::from(AppError::Internal);

        assert_eq!(
            serde_json::to_value(error).expect("public error should serialize"),
            json!({
                "code": "internal_error",
                "message": "Une erreur est survenue.",
                "field": null,
                "retryable": true
            })
        );
    }

    #[test]
    fn keeps_validation_messages_actionable() {
        let error = PublicError::from(AppError::Validation("Le nom est requis.".to_owned()));

        assert_eq!(error.code, "validation_failed");
        assert_eq!(error.message, "Le nom est requis.");
        assert!(!error.retryable);
    }

    #[test]
    fn explains_elevated_application_access_denial() {
        use crate::domain::ports::{ACCESS_DENIED_USER_MESSAGE, NativeError};

        let error = PublicError::from(NativeError::AccessDenied);
        assert_eq!(error.code, "access_denied");
        assert_eq!(error.message, ACCESS_DENIED_USER_MESSAGE);
        assert!(!error.retryable);
    }
}
