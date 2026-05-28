use axum::{
    Json,
    extract::{FromRequest, FromRequestParts, Path, Query, Request},
    http::request::Parts,
};
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::infra::error::AppError;

/// Extractor that deserializes a JSON body and validates it.
pub struct ValidatedJson<T>(pub T);

impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate + 'static,
    S: Send + Sync + 'static,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|e| AppError::BadRequest(e.to_string()))?;
        value.validate().map_err(AppError::from)?;
        Ok(ValidatedJson(value))
    }
}

macro_rules! validated_extractor {
    ($name:ident, $inner:ident, $trait:ident) => {
        pub struct $name<T>(pub T);

        impl<S, T> $trait<S> for $name<T>
        where
            T: DeserializeOwned + Validate + Send + 'static,
            S: Send + Sync + 'static,
        {
            type Rejection = AppError;

            async fn from_request_parts(
                parts: &mut Parts,
                state: &S,
            ) -> Result<Self, Self::Rejection> {
                let $inner(value) = $inner::<T>::from_request_parts(parts, state)
                    .await
                    .map_err(|e| AppError::BadRequest(e.to_string()))?;
                value.validate().map_err(AppError::from)?;
                Ok($name(value))
            }
        }
    };
}

validated_extractor!(ValidatedQuery, Query, FromRequestParts);
validated_extractor!(ValidatedPath, Path, FromRequestParts);
