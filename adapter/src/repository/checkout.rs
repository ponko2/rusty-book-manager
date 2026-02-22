use crate::database::{
    ConnectionSource,
    model::checkout::{CheckoutRow, CheckoutStateRow, ReturnedCheckoutRow},
};
use async_trait::async_trait;
use kernel::{
    model::{
        checkout::{
            Checkout, CheckoutState,
            event::{CreateCheckout, UpdateReturned},
        },
        id::{BookId, CheckoutId, UserId},
    },
    repository::checkout::CheckoutRepository,
};
use shared::error::{AppError, AppResult};

pub struct CheckoutRepositoryImpl<'t, 'm> {
    source: ConnectionSource<'t, 'm>,
}

impl<'t, 'm> CheckoutRepositoryImpl<'t, 'm> {
    pub fn new(source: impl Into<ConnectionSource<'t, 'm>>) -> Self {
        Self {
            source: source.into(),
        }
    }
}

#[async_trait]
impl<'t, 'm> CheckoutRepository for CheckoutRepositoryImpl<'t, 'm> {
    async fn delete_checkout(&self, checkout_id: CheckoutId) -> AppResult<()> {
        let mut conn = self.source.acquire().await?;
        let res = sqlx::query!(
            r#"
                DELETE FROM checkouts WHERE checkout_id = $1;
            "#,
            checkout_id as _,
        )
        .execute(&mut *conn)
        .await
        .map_err(AppError::SpecificOperationError)?;

        if res.rows_affected() < 1 {
            return Err(AppError::NoRowsAffectedError(
                "No checkout record has been deleted".into(),
            ));
        }

        Ok(())
    }

    async fn find_checkout_state(&self, book_id: BookId) -> AppResult<Option<CheckoutState>> {
        let mut conn = self.source.acquire().await?;
        let res = sqlx::query_as!(
            CheckoutStateRow,
            r#"
                SELECT
                    b.book_id,
                    c.checkout_id AS "checkout_id?: CheckoutId",
                    c.user_id AS "user_id?: UserId"
                FROM books AS b
                    LEFT OUTER JOIN checkouts AS c USING(book_id)
                WHERE book_id = $1;
            "#,
            book_id as _,
        )
        .fetch_optional(&mut *conn)
        .await
        .map_err(AppError::SpecificOperationError)?
        .map(CheckoutState::from);

        Ok(res)
    }

    async fn find_history_by_book_id(&self, book_id: BookId) -> AppResult<Vec<Checkout>> {
        let mut conn = self.source.acquire().await?;
        let checkout: Option<Checkout> = self.find_unreturned_by_book_id(book_id).await?;
        let mut checkout_histories: Vec<Checkout> = sqlx::query_as!(
            ReturnedCheckoutRow,
            r#"
                SELECT
                    rc.checkout_id,
                    rc.book_id,
                    rc.user_id,
                    rc.checked_out_at,
                    rc.returned_at,
                    b.title,
                    b.author,
                    b.isbn
                FROM returned_checkouts AS rc
                    INNER JOIN books AS b USING(book_id)
                WHERE rc.book_id = $1
                ORDER BY rc.checked_out_at DESC
            "#,
            book_id as _,
        )
        .fetch_all(&mut *conn)
        .await
        .map_err(AppError::SpecificOperationError)?
        .into_iter()
        .map(Checkout::try_from)
        .collect::<AppResult<_>>()?;

        if let Some(co) = checkout {
            checkout_histories.insert(0, co);
        }

        Ok(checkout_histories)
    }

    async fn find_unreturned_all(&self) -> AppResult<Vec<Checkout>> {
        let mut conn = self.source.acquire().await?;
        sqlx::query_as!(
            CheckoutRow,
            r#"
                SELECT
                    c.checkout_id,
                    c.book_id,
                    c.user_id,
                    c.checked_out_at,
                    b.title,
                    b.author,
                    b.isbn
                FROM checkouts AS c
                    INNER JOIN books AS b USING(book_id)
                ORDER BY c.checked_out_at ASC
                ;
            "#,
        )
        .fetch_all(&mut *conn)
        .await
        .map(|rows| rows.into_iter().map(Checkout::try_from).collect())
        .map_err(AppError::SpecificOperationError)?
    }

    async fn find_unreturned_by_user_id(&self, user_id: UserId) -> AppResult<Vec<Checkout>> {
        let mut conn = self.source.acquire().await?;
        sqlx::query_as!(
            CheckoutRow,
            r#"
                SELECT
                    c.checkout_id,
                    c.book_id,
                    c.user_id,
                    c.checked_out_at,
                    b.title,
                    b.author,
                    b.isbn
                FROM checkouts AS c
                    INNER JOIN books AS b USING(book_id)
                WHERE c.user_id = $1
                ORDER BY c.checked_out_at ASC
                ;
            "#,
            user_id as _,
        )
        .fetch_all(&mut *conn)
        .await
        .map(|rows| rows.into_iter().map(Checkout::try_from).collect())
        .map_err(AppError::SpecificOperationError)?
    }

    async fn insert_checkout(&self, event: &CreateCheckout) -> AppResult<()> {
        let mut conn = self.source.acquire().await?;
        let checkout_id = CheckoutId::new();
        let res = sqlx::query!(
            r#"
                INSERT INTO checkouts
                (checkout_id, book_id, user_id, checked_out_at)
                VALUES ($1, $2, $3, $4)
                ;
            "#,
            checkout_id as _,
            event.book_id as _,
            event.checked_out_by as _,
            event.checked_out_at,
        )
        .execute(&mut *conn)
        .await
        .map_err(AppError::SpecificOperationError)?;

        if res.rows_affected() < 1 {
            return Err(AppError::NoRowsAffectedError(
                "No checkout record has been created".into(),
            ));
        }

        Ok(())
    }

    async fn insert_returned_checkout(&self, event: &UpdateReturned) -> AppResult<()> {
        let mut conn = self.source.acquire().await?;
        let res = sqlx::query!(
            r#"
                INSERT INTO returned_checkouts
                (checkout_id, book_id, user_id, checked_out_at, returned_at)
                SELECT checkout_id, book_id, user_id, checked_out_at, $2
                FROM checkouts
                WHERE checkout_id = $1
                ;
            "#,
            event.checkout_id as _,
            event.returned_at,
        )
        .execute(&mut *conn)
        .await
        .map_err(AppError::SpecificOperationError)?;

        if res.rows_affected() < 1 {
            return Err(AppError::NoRowsAffectedError(
                "No returning record has been updated".into(),
            ));
        }

        Ok(())
    }
}

impl<'t, 'm> CheckoutRepositoryImpl<'t, 'm> {
    async fn find_unreturned_by_book_id(&self, book_id: BookId) -> AppResult<Option<Checkout>> {
        let mut conn = self.source.acquire().await?;
        let res = sqlx::query_as!(
            CheckoutRow,
            r#"
                SELECT
                    c.checkout_id,
                    c.book_id,
                    c.user_id,
                    c.checked_out_at,
                    b.title,
                    b.author,
                    b.isbn
                FROM checkouts AS c
                    INNER JOIN books AS b USING(book_id)
                WHERE c.book_id = $1
            "#,
            book_id as _,
        )
        .fetch_optional(&mut *conn)
        .await
        .map_err(AppError::SpecificOperationError)?
        .map(Checkout::try_from)
        .transpose()?;

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{database::ConnectionPool, redis::RedisClient, unit_of_work::UnitOfWorkScopeImpl};
    use chrono::Utc;
    use kernel::use_case::checkout::{CheckoutUseCase, CheckoutUseCaseImpl};
    use shared::config::RedisConfig;
    use std::{str::FromStr, sync::Arc};

    fn init_repo(
        pool: sqlx::PgPool,
    ) -> (
        CheckoutRepositoryImpl<'static, 'static>,
        CheckoutUseCaseImpl,
        UserId,
        UserId,
        BookId,
    ) {
        let repo = CheckoutRepositoryImpl::new(pool.clone());
        let use_case = CheckoutUseCaseImpl::new(Arc::new(UnitOfWorkScopeImpl::new(
            Arc::new(ConnectionPool::from(pool.clone())),
            Arc::new(
                RedisClient::new(&RedisConfig {
                    host: std::env::var("REDIS_HOST").unwrap(),
                    port: std::env::var("REDIS_PORT").unwrap().parse::<u16>().unwrap(),
                })
                .unwrap(),
            ),
            std::env::var("AUTH_TOKEN_TTL")
                .unwrap()
                .parse::<u64>()
                .unwrap(),
        )));

        let user_id1 = UserId::from_str("9582f9de-0fd1-4892-b20c-70139a7eb95b").unwrap();
        let user_id2 = UserId::from_str("050afe56-c3da-4448-8e4d-6f44007d2ca5").unwrap();
        let book_id1 = BookId::from_str("9890736e-a4e4-461a-a77d-eac3517ef11b").unwrap();

        (repo, use_case, user_id1, user_id2, book_id1)
    }

    #[sqlx::test(fixtures("common", "checkout"))]
    async fn test_checkout_and_return(pool: sqlx::PgPool) -> anyhow::Result<()> {
        let (repo, use_case, user_id1, user_id2, book_id1) = init_repo(pool);

        let res = repo.find_unreturned_by_user_id(user_id1).await?;
        assert!(res.is_empty());
        let res = repo.find_unreturned_by_user_id(user_id2).await?;
        assert!(res.is_empty());
        let co = repo.find_unreturned_by_book_id(book_id1).await?;
        assert!(co.is_none());

        {
            let res = use_case
                .checkout_book(CreateCheckout {
                    book_id: BookId::new(),
                    checked_out_by: user_id1,
                    checked_out_at: Utc::now(),
                })
                .await;
            assert!(matches!(res, Err(AppError::EntityNotFound(_))));
        }

        {
            use_case
                .checkout_book(CreateCheckout {
                    book_id: book_id1,
                    checked_out_by: user_id1,
                    checked_out_at: Utc::now(),
                })
                .await?;

            let co = repo.find_unreturned_by_book_id(book_id1).await?;
            assert!(
                matches!(co, Some(ref co) if co.book().book_id() == book_id1 && co.checked_out_by() == user_id1)
            );

            let res = use_case
                .checkout_book(CreateCheckout {
                    book_id: book_id1,
                    checked_out_by: user_id2,
                    checked_out_at: Utc::now(),
                })
                .await;
            assert!(res.is_err());

            let co = co.unwrap();

            let res = use_case
                .return_book(UpdateReturned {
                    checkout_id: co.id(),
                    book_id: BookId::new(),
                    returned_by: user_id1,
                    returned_at: Utc::now(),
                })
                .await;
            assert!(matches!(res, Err(AppError::EntityNotFound(_))));

            let res = use_case
                .return_book(UpdateReturned {
                    checkout_id: CheckoutId::new(),
                    book_id: book_id1,
                    returned_by: user_id1,
                    returned_at: Utc::now(),
                })
                .await;
            assert!(matches!(res, Err(AppError::UnprocessableEntity(_))));

            let res = use_case
                .return_book(UpdateReturned {
                    checkout_id: co.id(),
                    book_id: book_id1,
                    returned_by: user_id2,
                    returned_at: Utc::now(),
                })
                .await;
            assert!(matches!(res, Err(AppError::UnprocessableEntity(_))));

            use_case
                .return_book(UpdateReturned {
                    checkout_id: co.id(),
                    book_id: book_id1,
                    returned_by: user_id1,
                    returned_at: Utc::now(),
                })
                .await?;
        }

        Ok(())
    }

    #[sqlx::test(fixtures("common", "checkout"))]
    async fn test_checkout_list(pool: sqlx::PgPool) -> anyhow::Result<()> {
        let (repo, use_case, user_id1, user_id2, book_id1) = init_repo(pool);

        {
            use_case
                .checkout_book(CreateCheckout {
                    book_id: book_id1,
                    checked_out_by: user_id1,
                    checked_out_at: Utc::now(),
                })
                .await?;

            let co = repo.find_unreturned_by_book_id(book_id1).await?.unwrap();

            {
                let res = repo.find_unreturned_all().await?;
                assert_eq!(res.len(), 1);

                let res = repo.find_unreturned_by_user_id(user_id1).await?;
                assert_eq!(res.len(), 1);

                let res = repo.find_unreturned_by_user_id(user_id2).await?;
                assert_eq!(res.len(), 0);

                let res = repo.find_history_by_book_id(book_id1).await?;
                assert_eq!(res.len(), 1);
            }

            use_case
                .return_book(UpdateReturned {
                    checkout_id: co.id(),
                    book_id: book_id1,
                    returned_by: user_id1,
                    returned_at: Utc::now(),
                })
                .await?;

            {
                let res = repo.find_unreturned_all().await?;
                assert_eq!(res.len(), 0);

                let res = repo.find_unreturned_by_user_id(user_id1).await?;
                assert_eq!(res.len(), 0);

                let res = repo.find_unreturned_by_user_id(user_id2).await?;
                assert_eq!(res.len(), 0);

                let res = repo.find_history_by_book_id(book_id1).await?;
                assert_eq!(res.len(), 1);
            }
        }

        {
            use_case
                .checkout_book(CreateCheckout {
                    book_id: book_id1,
                    checked_out_by: user_id2,
                    checked_out_at: Utc::now(),
                })
                .await?;

            let co = repo.find_unreturned_by_book_id(book_id1).await?.unwrap();

            {
                let res = repo.find_unreturned_all().await?;
                assert_eq!(res.len(), 1);

                let res = repo.find_unreturned_by_user_id(user_id1).await?;
                assert_eq!(res.len(), 0);

                let res = repo.find_unreturned_by_user_id(user_id2).await?;
                assert_eq!(res.len(), 1);

                let res = repo.find_history_by_book_id(book_id1).await?;
                assert_eq!(res.len(), 2);
            }

            use_case
                .return_book(UpdateReturned {
                    checkout_id: co.id(),
                    book_id: book_id1,
                    returned_by: user_id2,
                    returned_at: Utc::now(),
                })
                .await?;

            {
                let res = repo.find_unreturned_all().await?;
                assert_eq!(res.len(), 0);

                let res = repo.find_unreturned_by_user_id(user_id1).await?;
                assert_eq!(res.len(), 0);

                let res = repo.find_unreturned_by_user_id(user_id2).await?;
                assert_eq!(res.len(), 0);

                let res = repo.find_history_by_book_id(book_id1).await?;
                assert_eq!(res.len(), 2);
            }
        }

        Ok(())
    }
}
