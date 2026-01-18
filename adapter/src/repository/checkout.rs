use crate::database::{
    ConnectionPool,
    model::checkout::{CheckoutRow, CheckoutStateRow, ReturnedCheckoutRow},
};
use async_trait::async_trait;
use derive_new::new;
use kernel::model::checkout::{
    Checkout,
    event::{CreateCheckout, UpdateReturned},
};
use kernel::{
    model::id::{BookId, CheckoutId, UserId},
    repository::checkout::CheckoutRepository,
};
use shared::error::{AppError, AppResult};

#[derive(new)]
pub struct CheckoutRepositoryImpl {
    db: ConnectionPool,
}

#[async_trait]
impl CheckoutRepository for CheckoutRepositoryImpl {
    async fn create(&self, event: CreateCheckout) -> AppResult<()> {
        let mut tx = self.db.begin().await?;

        self.set_transaction_serializable(&mut tx).await?;

        {
            let res = sqlx::query_as!(
                CheckoutStateRow,
                r#"
                    SELECT
                        b.book_id,
                        c.checkout_id AS "checkout_id?: CheckoutId",
                        NULL AS "user_id?: UserId"
                    FROM books AS b
                        LEFT OUTER JOIN checkouts AS c USING(book_id)
                    WHERE book_id = $1;
                "#,
                event.book_id as _
            )
            .fetch_optional(&mut *tx)
            .await
            .map_err(AppError::SpecificOperationError)?;

            match res {
                None => {
                    return Err(AppError::EntityNotFound(format!(
                        " 書籍（{}）が見つかりませんでした。",
                        event.book_id
                    )));
                }
                Some(CheckoutStateRow {
                    checkout_id: Some(_),
                    ..
                }) => {
                    return Err(AppError::UnprocessableEntity(format!(
                        " 書籍（{}）に対する貸出が既に存在します。",
                        event.book_id
                    )));
                }
                _ => {}
            }
        }

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
        .execute(&mut *tx)
        .await
        .map_err(AppError::SpecificOperationError)?;

        if res.rows_affected() < 1 {
            return Err(AppError::NoRowsAffectedError(
                "No checkout record has been created".into(),
            ));
        }

        tx.commit().await.map_err(AppError::TransactionError)?;

        Ok(())
    }

    async fn update_returned(&self, event: UpdateReturned) -> AppResult<()> {
        let mut tx = self.db.begin().await?;

        self.set_transaction_serializable(&mut tx).await?;

        {
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
                event.book_id as _,
            )
            .fetch_optional(&mut *tx)
            .await
            .map_err(AppError::SpecificOperationError)?;

            match res {
                None => {
                    return Err(AppError::EntityNotFound(format!(
                        " 書籍（{}）が見つかりませんでした。",
                        event.book_id
                    )));
                }
                Some(CheckoutStateRow {
                    checkout_id: Some(c),
                    user_id: Some(u),
                    ..
                }) if (c, u) != (event.checkout_id, event.returned_by) => {
                    return Err(AppError::UnprocessableEntity(format!(
                        " 指定の貸出（ID（{}）, ユーザー（{}）, 書籍（{}））は返却できません。",
                        event.checkout_id, event.returned_by, event.book_id
                    )));
                }
                _ => {}
            }
        }

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
        .execute(&mut *tx)
        .await
        .map_err(AppError::SpecificOperationError)?;

        if res.rows_affected() < 1 {
            return Err(AppError::NoRowsAffectedError(
                "No returning record has been updated".into(),
            ));
        }

        let res = sqlx::query!(
            r#"
                DELETE FROM checkouts WHERE checkout_id = $1;
            "#,
            event.checkout_id as _,
        )
        .execute(&mut *tx)
        .await
        .map_err(AppError::SpecificOperationError)?;

        if res.rows_affected() < 1 {
            return Err(AppError::NoRowsAffectedError(
                "No checkout record has been deleted".into(),
            ));
        }

        tx.commit().await.map_err(AppError::TransactionError)?;

        Ok(())
    }

    async fn find_unreturned_all(&self) -> AppResult<Vec<Checkout>> {
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
        .fetch_all(self.db.inner_ref())
        .await
        .map(|rows| rows.into_iter().map(Checkout::from).collect())
        .map_err(AppError::SpecificOperationError)
    }

    async fn find_unreturned_by_user_id(&self, user_id: UserId) -> AppResult<Vec<Checkout>> {
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
            user_id as _
        )
        .fetch_all(self.db.inner_ref())
        .await
        .map(|rows| rows.into_iter().map(Checkout::from).collect())
        .map_err(AppError::SpecificOperationError)
    }

    async fn find_history_by_book_id(&self, book_id: BookId) -> AppResult<Vec<Checkout>> {
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
            book_id as _
        )
        .fetch_all(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?
        .into_iter()
        .map(Checkout::from)
        .collect();

        if let Some(co) = checkout {
            checkout_histories.insert(0, co);
        }

        Ok(checkout_histories)
    }
}

impl CheckoutRepositoryImpl {
    async fn set_transaction_serializable(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> AppResult<()> {
        sqlx::query!("SET TRANSACTION ISOLATION LEVEL SERIALIZABLE")
            .execute(&mut **tx)
            .await
            .map_err(AppError::SpecificOperationError)?;
        Ok(())
    }

    async fn find_unreturned_by_book_id(&self, book_id: BookId) -> AppResult<Option<Checkout>> {
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
        .fetch_optional(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?
        .map(Checkout::from);

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use kernel::model::checkout::CheckoutBook;

    use super::*;
    use std::str::FromStr;

    fn init_repo(pool: sqlx::PgPool) -> (CheckoutRepositoryImpl, UserId, UserId, BookId) {
        let repo = CheckoutRepositoryImpl::new(ConnectionPool::new(pool));

        let user_id1 = UserId::from_str("9582f9de-0fd1-4892-b20c-70139a7eb95b").unwrap();
        let user_id2 = UserId::from_str("050afe56-c3da-4448-8e4d-6f44007d2ca5").unwrap();
        let book_id1 = BookId::from_str("9890736e-a4e4-461a-a77d-eac3517ef11b").unwrap();

        (repo, user_id1, user_id2, book_id1)
    }

    #[sqlx::test(fixtures("common", "checkout"))]
    async fn test_checkout_and_return(pool: sqlx::PgPool) -> anyhow::Result<()> {
        let (repo, user_id1, user_id2, book_id1) = init_repo(pool);

        let res = repo.find_unreturned_by_user_id(user_id1).await?;
        assert!(res.is_empty());
        let res = repo.find_unreturned_by_user_id(user_id2).await?;
        assert!(res.is_empty());
        let co = repo.find_unreturned_by_book_id(book_id1).await?;
        assert!(co.is_none());

        {
            let res = repo
                .create(CreateCheckout {
                    book_id: BookId::new(),
                    checked_out_by: user_id1,
                    checked_out_at: Utc::now(),
                })
                .await;
            assert!(matches!(res, Err(AppError::EntityNotFound(_))));
        }

        {
            repo.create(CreateCheckout {
                book_id: book_id1,
                checked_out_by: user_id1,
                checked_out_at: Utc::now(),
            })
            .await?;

            let co = repo.find_unreturned_by_book_id(book_id1).await?;
            assert!(
                matches!(co, Some(Checkout{checked_out_by,book:CheckoutBook{book_id,..},..}) if book_id == book_id1 && checked_out_by == user_id1)
            );

            let res = repo
                .create(CreateCheckout {
                    book_id: book_id1,
                    checked_out_by: user_id2,
                    checked_out_at: Utc::now(),
                })
                .await;
            assert!(res.is_err());

            let co = co.unwrap();

            let res = repo
                .update_returned(UpdateReturned {
                    checkout_id: co.id,
                    book_id: BookId::new(),
                    returned_by: user_id1,
                    returned_at: Utc::now(),
                })
                .await;
            assert!(matches!(res, Err(AppError::EntityNotFound(_))));

            let res = repo
                .update_returned(UpdateReturned {
                    checkout_id: CheckoutId::new(),
                    book_id: book_id1,
                    returned_by: user_id1,
                    returned_at: Utc::now(),
                })
                .await;
            assert!(matches!(res, Err(AppError::UnprocessableEntity(_))));

            let res = repo
                .update_returned(UpdateReturned {
                    checkout_id: co.id,
                    book_id: book_id1,
                    returned_by: user_id2,
                    returned_at: Utc::now(),
                })
                .await;
            assert!(matches!(res, Err(AppError::UnprocessableEntity(_))));

            repo.update_returned(UpdateReturned {
                checkout_id: co.id,
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
        let (repo, user_id1, user_id2, book_id1) = init_repo(pool);

        {
            repo.create(CreateCheckout {
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

            repo.update_returned(UpdateReturned {
                checkout_id: co.id,
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
            repo.create(CreateCheckout {
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

            repo.update_returned(UpdateReturned {
                checkout_id: co.id,
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
