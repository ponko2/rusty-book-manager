use crate::{
    database::model::auth::{AuthorizationKey, AuthorizedUserId, from},
    redis::RedisClient,
};
use async_trait::async_trait;
use kernel::{
    model::{
        auth::{AccessToken, event::CreateToken},
        id::UserId,
    },
    repository::auth::AuthRepository,
};
use shared::error::AppResult;
use std::sync::Arc;

pub struct AuthRepositoryImpl {
    kv: Arc<RedisClient>,
    ttl: u64,
}

impl AuthRepositoryImpl {
    pub fn new(kv: Arc<RedisClient>, ttl: u64) -> Self {
        Self { kv, ttl }
    }
}

#[async_trait]
impl AuthRepository for AuthRepositoryImpl {
    async fn create_token(&self, event: CreateToken) -> AppResult<AccessToken> {
        let (key, value) = from(event);
        self.kv.set_ex(&key, &value, self.ttl).await?;
        Ok(key.into())
    }

    async fn delete_token(&self, access_token: AccessToken) -> AppResult<()> {
        let key: AuthorizationKey = access_token.into();
        self.kv.delete(&key).await
    }

    async fn fetch_user_id_from_token(
        &self,
        access_token: &AccessToken,
    ) -> AppResult<Option<UserId>> {
        let key: AuthorizationKey = access_token.into();
        self.kv
            .get(&key)
            .await
            .map(|x| x.map(AuthorizedUserId::into_inner))
    }
}
