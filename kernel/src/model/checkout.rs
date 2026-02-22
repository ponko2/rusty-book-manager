use crate::model::{
    id::{BookId, CheckoutId, UserId},
    value::{BookAuthor, BookIsbn, BookTitle},
};
use chrono::{DateTime, Utc};

pub mod event;

#[derive(Debug)]
pub struct Checkout {
    id: CheckoutId,
    checked_out_by: UserId,
    checked_out_at: DateTime<Utc>,
    returned_at: Option<DateTime<Utc>>,
    book: CheckoutBook,
}

impl Checkout {
    pub fn new(
        id: CheckoutId,
        checked_out_by: UserId,
        checked_out_at: DateTime<Utc>,
        returned_at: Option<DateTime<Utc>>,
        book: CheckoutBook,
    ) -> Self {
        Self {
            id,
            checked_out_by,
            checked_out_at,
            returned_at,
            book,
        }
    }

    pub fn id(&self) -> CheckoutId {
        self.id
    }

    pub fn checked_out_by(&self) -> UserId {
        self.checked_out_by
    }

    pub fn checked_out_at(&self) -> DateTime<Utc> {
        self.checked_out_at
    }

    pub fn returned_at(&self) -> Option<DateTime<Utc>> {
        self.returned_at
    }

    pub fn book(&self) -> &CheckoutBook {
        &self.book
    }
}

#[derive(Debug, Clone)]
pub struct CheckoutBook {
    book_id: BookId,
    title: BookTitle,
    author: BookAuthor,
    isbn: BookIsbn,
}

impl CheckoutBook {
    pub fn new(book_id: BookId, title: BookTitle, author: BookAuthor, isbn: BookIsbn) -> Self {
        Self {
            book_id,
            title,
            author,
            isbn,
        }
    }

    pub fn book_id(&self) -> BookId {
        self.book_id
    }

    pub fn title(&self) -> &BookTitle {
        &self.title
    }

    pub fn author(&self) -> &BookAuthor {
        &self.author
    }

    pub fn isbn(&self) -> &BookIsbn {
        &self.isbn
    }

    pub fn into_parts(self) -> (BookId, BookTitle, BookAuthor, BookIsbn) {
        (self.book_id, self.title, self.author, self.isbn)
    }
}

#[derive(Debug)]
pub struct CheckoutState {
    pub book_id: BookId,
    pub checkout_id: Option<CheckoutId>,
    pub user_id: Option<UserId>,
}
