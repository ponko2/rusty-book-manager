use axum::{RequestPartsExt, async_trait, extract::FromRequestParts, http::request::Parts};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use kernel::model::{auth::AccessToken, id::UserId, role::Role, user::User};
use registry::AppRegistry;
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
