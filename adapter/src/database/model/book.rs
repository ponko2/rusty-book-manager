use chrono::{DateTime, Utc};
use kernel::model::{
    book::{Book, Checkout},
    id::{BookId, CheckoutId, UserId},
    user::{BookOwner, CheckoutUser},
};
use shared::error::{AppError, AppResult};

pub struct BookRow {
    pub book_id: BookId,
    pub title: String,
    pub author: String,
    pub isbn: String,
    pub description: String,
    pub owned_by: UserId,
    pub owner_name: String,
}

impl BookRow {
    pub fn try_into_book(self, checkout: Option<Checkout>) -> AppResult<Book> {
        let BookRow {
            book_id,
            title,
            author,
            isbn,
            description,
            owned_by,
            owner_name,
        } = self;
        Ok(Book::new(
            book_id,
            title.parse()?,
            author.parse()?,
            isbn.parse()?,
            description.parse()?,
            BookOwner::new(owned_by, owner_name.parse()?),
            checkout,
        ))
    }
}

pub struct PaginatedBookRow {
    pub total: i64,
    pub id: BookId,
}

pub struct BookCheckoutRow {
    pub checkout_id: CheckoutId,
    pub book_id: BookId,
    pub user_id: UserId,
    pub user_name: String,
    pub checked_out_at: DateTime<Utc>,
}

impl TryFrom<BookCheckoutRow> for Checkout {
    type Error = AppError;

    fn try_from(value: BookCheckoutRow) -> Result<Self, Self::Error> {
        let BookCheckoutRow {
            checkout_id,
            book_id: _,
            user_id,
            user_name,
            checked_out_at,
        } = value;
        Ok(Checkout::new(
            checkout_id,
            CheckoutUser::new(user_id, user_name.parse()?),
            checked_out_at,
        ))
    }
}
