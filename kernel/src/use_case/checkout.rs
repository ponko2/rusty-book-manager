use crate::{
    model::{
        checkout::{
            Checkout, CheckoutState,
            event::{CreateCheckout, UpdateReturned},
        },
        id::BookId,
    },
    unit_of_work::checkout::CheckoutUnitOfWorkScope,
};
use async_trait::async_trait;
use shared::error::{AppError, AppResult};
use std::sync::Arc;

#[mockall::automock]
#[async_trait]
pub trait CheckoutUseCase: Send + Sync {
    async fn checkout_book(&self, event: CreateCheckout) -> AppResult<()>;
    async fn checkout_history(&self, book_id: BookId) -> AppResult<Vec<Checkout>>;
    async fn return_book(&self, event: UpdateReturned) -> AppResult<()>;
    async fn show_checked_out_list(&self) -> AppResult<Vec<Checkout>>;
}

pub struct CheckoutUseCaseImpl {
    scope: Arc<dyn CheckoutUnitOfWorkScope>,
}

impl CheckoutUseCaseImpl {
    pub fn new(scope: Arc<dyn CheckoutUnitOfWorkScope>) -> Self {
        Self { scope }
    }
}

#[async_trait]
impl CheckoutUseCase for CheckoutUseCaseImpl {
    async fn checkout_book(&self, event: CreateCheckout) -> AppResult<()> {
        let uow = self.scope.begin_serializable().await?;

        {
            let checkout_repository = uow.checkout_repository();
            let res = checkout_repository
                .find_checkout_state(event.book_id)
                .await?;

            match res {
                None => {
                    return Err(AppError::EntityNotFound(format!(
                        " 書籍（{}）が見つかりませんでした。",
                        event.book_id
                    )));
                }
                Some(CheckoutState {
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

            checkout_repository.insert_checkout(&event).await?;
        }

        uow.commit().await
    }

    async fn checkout_history(&self, book_id: BookId) -> AppResult<Vec<Checkout>> {
        let uow = self.scope.begin().await?;
        uow.checkout_repository()
            .find_history_by_book_id(book_id)
            .await
    }

    async fn return_book(&self, event: UpdateReturned) -> AppResult<()> {
        let uow = self.scope.begin_serializable().await?;

        {
            let checkout_repository = uow.checkout_repository();
            let res = checkout_repository
                .find_checkout_state(event.book_id)
                .await?;

            match res {
                None => {
                    return Err(AppError::EntityNotFound(format!(
                        " 書籍（{}）が見つかりませんでした。",
                        event.book_id
                    )));
                }
                Some(CheckoutState {
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

            checkout_repository.insert_returned_checkout(&event).await?;
            checkout_repository
                .delete_checkout(event.checkout_id)
                .await?;
        }

        uow.commit().await
    }

    async fn show_checked_out_list(&self) -> AppResult<Vec<Checkout>> {
        let uow = self.scope.begin().await?;
        uow.checkout_repository().find_unreturned_all().await
    }
}
