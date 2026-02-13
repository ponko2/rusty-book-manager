use crate::database::{
    ConnectionSource,
    model::user::{UserItem, UserRow},
};
use async_trait::async_trait;
use kernel::{
    model::{
        id::UserId,
        role::Role,
        user::{
            User,
            event::{CreateUser, DeleteUser, UpdateUserPassword, UpdateUserRole},
        },
    },
    repository::user::UserRepository,
};
use shared::error::{AppError, AppResult};

pub struct UserRepositoryImpl<'t, 'm> {
    source: ConnectionSource<'t, 'm>,
}

impl<'t, 'm> UserRepositoryImpl<'t, 'm> {
    pub fn new(source: impl Into<ConnectionSource<'t, 'm>>) -> Self {
        Self {
            source: source.into(),
        }
    }
}

#[async_trait]
impl<'t, 'm> UserRepository for UserRepositoryImpl<'t, 'm> {
    async fn create(&self, event: CreateUser) -> AppResult<User> {
        let mut conn = self.source.acquire().await?;
        let user_id = UserId::new();
        let hashed_password = hash_password(&event.password)?;
        let role = Role::User;
        let res = sqlx::query!(
            r#"
                INSERT INTO users(user_id, name, email, password_hash, role_id)
                SELECT $1, $2, $3, $4, role_id FROM roles WHERE name = $5;
            "#,
            user_id as _,
            event.name,
            event.email,
            hashed_password,
            role.as_ref()
        )
        .execute(&mut *conn)
        .await
        .map_err(AppError::SpecificOperationError)?;
        if res.rows_affected() < 1 {
            return Err(AppError::NoRowsAffectedError(
                "No user has been created".into(),
            ));
        }
        Ok(User {
            id: user_id,
            name: event.name,
            email: event.email,
            role,
        })
    }

    async fn delete(&self, event: DeleteUser) -> AppResult<()> {
        let mut conn = self.source.acquire().await?;
        let res = sqlx::query!(
            r#"
                DELETE FROM users
                WHERE user_id = $1
            "#,
            event.user_id as _
        )
        .execute(&mut *conn)
        .await
        .map_err(AppError::SpecificOperationError)?;
        if res.rows_affected() < 1 {
            return Err(AppError::EntityNotFound("Specified user not found".into()));
        }
        Ok(())
    }

    async fn find_all(&self) -> AppResult<Vec<User>> {
        let mut conn = self.source.acquire().await?;
        let users = sqlx::query_as!(
            UserRow,
            r#"
                SELECT
                    u.user_id,
                    u.name,
                    u.email,
                    r.name as role_name,
                    u.created_at,
                    u.updated_at
                FROM users AS u
                    INNER JOIN roles AS r USING(role_id)
                ORDER BY u.created_at DESC;
            "#,
        )
        .fetch_all(&mut *conn)
        .await
        .map_err(AppError::SpecificOperationError)?
        .into_iter()
        .filter_map(|row| User::try_from(row).ok())
        .collect();
        Ok(users)
    }

    async fn find_current_user(&self, current_user_id: UserId) -> AppResult<Option<User>> {
        let mut conn = self.source.acquire().await?;
        let row = sqlx::query_as!(
            UserRow,
            r#"
                SELECT
                    u.user_id,
                    u.name,
                    u.email,
                    r.name as role_name,
                    u.created_at,
                    u.updated_at
                FROM users AS u
                    INNER JOIN roles AS r USING(role_id)
                WHERE u.user_id = $1
            "#,
            current_user_id as _,
        )
        .fetch_optional(&mut *conn)
        .await
        .map_err(AppError::SpecificOperationError)?;
        match row {
            Some(r) => Ok(Some(User::try_from(r)?)),
            None => Ok(None),
        }
    }

    async fn find_password_hash_by_email(&self, email: &str) -> AppResult<(UserId, String)> {
        let mut conn = self.source.acquire().await?;
        let UserItem {
            user_id,
            password_hash,
        } = sqlx::query_as!(
            UserItem,
            r#"
                SELECT user_id, password_hash FROM users
                WHERE email = $1;
            "#,
            email,
        )
        .fetch_one(&mut *conn)
        .await
        .map_err(AppError::SpecificOperationError)?;
        Ok((user_id, password_hash))
    }

    async fn find_password_hash_by_user_id(&self, user_id: UserId) -> AppResult<String> {
        let mut conn = self.source.acquire().await?;
        let res = sqlx::query!(
            r#"
                SELECT password_hash FROM users WHERE user_id = $1;
            "#,
            user_id as _
        )
        .fetch_one(&mut *conn)
        .await
        .map_err(AppError::SpecificOperationError)?;
        Ok(res.password_hash)
    }

    async fn update_password(&self, event: UpdateUserPassword) -> AppResult<()> {
        let mut conn = self.source.acquire().await?;
        let password_hash = hash_password(&event.new_password)?;
        sqlx::query!(
            r#"
                UPDATE users SET password_hash = $2 WHERE user_id = $1;
            "#,
            event.user_id as _,
            password_hash,
        )
        .execute(&mut *conn)
        .await
        .map_err(AppError::SpecificOperationError)?;
        Ok(())
    }

    async fn update_role(&self, event: UpdateUserRole) -> AppResult<()> {
        let mut conn = self.source.acquire().await?;
        let res = sqlx::query!(
            r#"
                UPDATE users
                SET role_id = (
                    SELECT role_id FROM roles WHERE name = $2
                )
                WHERE user_id = $1
            "#,
            event.user_id as _,
            event.role.as_ref()
        )
        .execute(&mut *conn)
        .await
        .map_err(AppError::SpecificOperationError)?;
        if res.rows_affected() < 1 {
            return Err(AppError::EntityNotFound("Specified user not found".into()));
        }
        Ok(())
    }
}

fn hash_password(password: &str) -> AppResult<String> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST).map_err(AppError::from)
}

#[cfg(test)]
mod tests {
    use super::UserRepositoryImpl;
    use kernel::{
        model::{
            id::UserId,
            role::Role,
            user::{
                User,
                event::{CreateUser, DeleteUser, UpdateUserPassword, UpdateUserRole},
            },
        },
        repository::user::UserRepository,
    };
    use std::str::FromStr;

    #[sqlx::test(fixtures("common"))]
    async fn test_find_current_user(pool: sqlx::PgPool) -> anyhow::Result<()> {
        let repo = UserRepositoryImpl::new(pool);
        let current_user_id = UserId::from_str("5b4c96ac-316a-4bee-8e69-cac5eb84ff4c")?;
        let me = repo.find_current_user(current_user_id).await?;
        assert!(me.is_some());
        assert_eq!(
            me,
            Some(User {
                id: UserId::from_str("5b4c96ac-316a-4bee-8e69-cac5eb84ff4c")?,
                email: "eleazar.fig@example.com".into(),
                name: "Eleazar Fig".into(),
                role: Role::Admin,
            })
        );

        Ok(())
    }

    #[sqlx::test(fixtures("common"))]
    async fn test_users(pool: sqlx::PgPool) -> anyhow::Result<()> {
        let repo = UserRepositoryImpl::new(pool.clone());

        let event = CreateUser {
            name: "Test".into(),
            email: "test@example.com".into(),
            password: "dummy".into(),
        };
        let user = repo.create(event).await?;

        {
            let event = UpdateUserPassword {
                user_id: user.id,
                current_password: "dummy".into(),
                new_password: "new_password".into(),
            };
            repo.update_password(event).await?;

            let event = UpdateUserRole {
                user_id: user.id,
                role: Role::Admin,
            };
            repo.update_role(event).await?;
        }

        let user_found = repo.find_current_user(user.id).await?;
        assert_eq!(user_found.unwrap().id, user.id);

        let users = repo.find_all().await?;
        assert!(!users.is_empty());

        {
            let event = DeleteUser { user_id: user.id };
            repo.delete(event).await?;
        }

        let user_deleted = repo.find_current_user(user.id).await?;
        assert!(user_deleted.is_none());

        Ok(())
    }
}
