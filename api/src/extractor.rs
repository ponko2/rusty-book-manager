use axum::{
    Json, RequestPartsExt, async_trait,
    body::Body,
    extract::{
        FromRequest, FromRequestParts, Query,
        rejection::{JsonRejection, QueryRejection},
    },
    http::{Request, request::Parts},
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use garde::Validate;
use kernel::model::{auth::AccessToken, id::UserId, role::Role, user::User};
use registry::AppRegistry;
use serde::de::DeserializeOwned;
use shared::error::AppError;

pub struct AuthorizedUser {
    pub access_token: AccessToken,
    pub user: User,
}

impl AuthorizedUser {
    pub fn id(&self) -> UserId {
        self.user.id()
    }
    pub fn is_admin(&self) -> bool {
        self.user.role() == &Role::Admin
    }
}

#[async_trait]
impl FromRequestParts<AppRegistry> for AuthorizedUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        registry: &AppRegistry,
    ) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AppError::UnauthorizedError)?;
        let access_token = AccessToken(bearer.token().to_string());

        let user = registry
            .auth_use_case()
            .find_authorized_user(&access_token)
            .await?;

        Ok(Self { access_token, user })
    }
}

pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate<Context = ()>,
    S: Send + Sync,
    Json<T>: FromRequest<S, Rejection = JsonRejection>,
{
    type Rejection = AppError;

    async fn from_request(req: Request<Body>, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state).await?;
        value.validate(&())?;
        Ok(ValidatedJson(value))
    }
}

pub struct ValidatedQuery<T>(pub T);

#[async_trait]
impl<T, S> FromRequestParts<S> for ValidatedQuery<T>
where
    T: DeserializeOwned + Validate<Context = ()>,
    S: Send + Sync,
    Query<T>: FromRequestParts<S, Rejection = QueryRejection>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Query(value) = Query::<T>::from_request_parts(parts, state).await?;
        value.validate(&())?;
        Ok(ValidatedQuery(value))
    }
}
