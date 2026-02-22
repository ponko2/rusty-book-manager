use crate::model::{
    checkout::{
        Checkout, CheckoutState,
        event::{CreateCheckout, UpdateReturned},
    },
    id::{BookId, CheckoutId, UserId},
};
use async_trait::async_trait;
use shared::error::AppResult;

#[mockall::automock]
#[async_trait]
pub trait CheckoutRepository: Send + Sync {
    async fn delete_checkout(&self, checkout_id: CheckoutId) -> AppResult<()>;
    async fn find_checkout_state(&self, book_id: BookId) -> AppResult<Option<CheckoutState>>;
    async fn find_history_by_book_id(&self, book_id: BookId) -> AppResult<Vec<Checkout>>;
    async fn find_unreturned_all(&self) -> AppResult<Vec<Checkout>>;
    async fn find_unreturned_by_user_id(&self, user_id: UserId) -> AppResult<Vec<Checkout>>;
    async fn insert_checkout(&self, event: &CreateCheckout) -> AppResult<()>;
    async fn insert_returned_checkout(&self, event: &UpdateReturned) -> AppResult<()>;
}
