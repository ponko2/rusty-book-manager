use kernel::model::{
    checkout::{Checkout, CheckoutBook, CheckoutState},
    id::{BookId, CheckoutId, UserId},
};
use shared::error::AppError;
use sqlx::types::chrono::{DateTime, Utc};

pub struct CheckoutStateRow {
    pub book_id: BookId,
    pub checkout_id: Option<CheckoutId>,
    pub user_id: Option<UserId>,
}

impl From<CheckoutStateRow> for CheckoutState {
    fn from(value: CheckoutStateRow) -> Self {
        let CheckoutStateRow {
            book_id,
            checkout_id,
            user_id,
        } = value;
        CheckoutState {
            book_id,
            checkout_id,
            user_id,
        }
    }
}

pub struct CheckoutRow {
    pub checkout_id: CheckoutId,
    pub book_id: BookId,
    pub user_id: UserId,
    pub checked_out_at: DateTime<Utc>,
    pub title: String,
    pub author: String,
    pub isbn: String,
}

impl TryFrom<CheckoutRow> for Checkout {
    type Error = AppError;

    fn try_from(value: CheckoutRow) -> Result<Self, Self::Error> {
        let CheckoutRow {
            checkout_id,
            book_id,
            user_id,
            checked_out_at,
            title,
            author,
            isbn,
        } = value;
        Ok(Checkout::new(
            checkout_id,
            user_id,
            checked_out_at,
            None,
            CheckoutBook::new(book_id, title.parse()?, author.parse()?, isbn.parse()?),
        ))
    }
}

pub struct ReturnedCheckoutRow {
    pub checkout_id: CheckoutId,
    pub book_id: BookId,
    pub user_id: UserId,
    pub checked_out_at: DateTime<Utc>,
    pub returned_at: DateTime<Utc>,
    pub title: String,
    pub author: String,
    pub isbn: String,
}

impl TryFrom<ReturnedCheckoutRow> for Checkout {
    type Error = AppError;

    fn try_from(value: ReturnedCheckoutRow) -> Result<Self, Self::Error> {
        let ReturnedCheckoutRow {
            checkout_id,
            book_id,
            user_id,
            checked_out_at,
            returned_at,
            title,
            author,
            isbn,
        } = value;
        Ok(Checkout::new(
            checkout_id,
            user_id,
            checked_out_at,
            Some(returned_at),
            CheckoutBook::new(book_id, title.parse()?, author.parse()?, isbn.parse()?),
        ))
    }
}
