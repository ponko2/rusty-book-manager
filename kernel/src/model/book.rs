use crate::model::{
    id::{BookId, CheckoutId},
    user::{BookOwner, CheckoutUser},
    value::{BookAuthor, BookDescription, BookIsbn, BookTitle},
};
use chrono::{DateTime, Utc};
use derive_new::new;
use shared::error::AppError;

pub mod event;

#[derive(Debug, PartialEq, Eq)]
pub struct Book {
    id: BookId,
    title: BookTitle,
    author: BookAuthor,
    isbn: BookIsbn,
    description: BookDescription,
    owner: BookOwner,
    checkout: Option<Checkout>,
}

impl Book {
    pub fn new(
        id: BookId,
        title: BookTitle,
        author: BookAuthor,
        isbn: BookIsbn,
        description: BookDescription,
        owner: BookOwner,
        checkout: Option<Checkout>,
    ) -> Self {
        Self {
            id,
            title,
            author,
            isbn,
            description,
            owner,
            checkout,
        }
    }

    pub fn id(&self) -> BookId {
        self.id
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

    pub fn description(&self) -> &BookDescription {
        &self.description
    }

    pub fn owner(&self) -> &BookOwner {
        &self.owner
    }

    pub fn checkout(&self) -> Option<&Checkout> {
        self.checkout.as_ref()
    }

    pub fn into_parts(
        self,
    ) -> (
        BookId,
        BookTitle,
        BookAuthor,
        BookIsbn,
        BookDescription,
        BookOwner,
        Option<Checkout>,
    ) {
        (
            self.id,
            self.title,
            self.author,
            self.isbn,
            self.description,
            self.owner,
            self.checkout,
        )
    }
}

impl TryFrom<(event::CreateBook, BookOwner)> for Book {
    type Error = AppError;

    fn try_from(value: (event::CreateBook, BookOwner)) -> Result<Self, Self::Error> {
        let (event, owner) = value;
        Ok(Self::new(
            BookId::new(),
            event.title,
            event.author,
            event.isbn,
            event.description,
            owner,
            None,
        ))
    }
}

#[derive(Debug)]
pub struct BookListOptions {
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, new)]
pub struct Checkout {
    checkout_id: CheckoutId,
    checked_out_by: CheckoutUser,
    checked_out_at: DateTime<Utc>,
}

impl Checkout {
    pub fn id(&self) -> CheckoutId {
        self.checkout_id
    }

    pub fn checked_out_by(&self) -> &CheckoutUser {
        &self.checked_out_by
    }

    pub fn checked_out_at(&self) -> DateTime<Utc> {
        self.checked_out_at
    }

    pub fn into_parts(self) -> (CheckoutId, CheckoutUser, DateTime<Utc>) {
        (self.checkout_id, self.checked_out_by, self.checked_out_at)
    }
}
