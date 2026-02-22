use crate::{
    extractor::AuthorizedUser,
    model::auth::{AccessTokenResponse, LoginRequest},
};
use axum::{Json, extract::State, http::StatusCode};
use registry::AppRegistry;
use shared::error::AppResult;

#[cfg_attr(
    debug_assertions,
    utoipa::path(
        post,
        path="/auth/login",
        request_body = LoginRequest,
        responses(
            (status = 200, description = "ログインに成功した場合。", body = AccessTokenResponse),
            (status = 400, description = "リクエストの内容に問題があった場合。"),
            (status = 403, description = "ログイン認証が通らなかった場合。ユーザーIDないしはパスワードに誤りがある可能性があります。")
        )
    )
)]
#[tracing::instrument(
    skip(registry, req),
    fields(
        email_address = %req.email
    )
)]
pub async fn login(
    State(registry): State<AppRegistry>,
    Json(req): Json<LoginRequest>,
) -> AppResult<Json<AccessTokenResponse>> {
    let (user_id, access_token) = registry
        .auth_use_case()
        .login(&req.email, &req.password)
        .await?;
    Ok(Json(AccessTokenResponse {
        user_id,
        access_token: access_token.0,
    }))
}

#[cfg_attr(
    debug_assertions,
    utoipa::path(
        post,
        path="/auth/logout",
        responses(
            (status = 204, description = "ログアウトに成功した場合。"),
        )
    )
)]
#[tracing::instrument(
    skip(registry, user),
    fields(
        user_id = %user.user.id().to_string(),
        user_name = %user.user.name().to_string()
    )
)]
pub async fn logout(
    user: AuthorizedUser,
    State(registry): State<AppRegistry>,
) -> AppResult<StatusCode> {
    registry.auth_use_case().logout(user.access_token).await?;
    Ok(StatusCode::NO_CONTENT)
}
