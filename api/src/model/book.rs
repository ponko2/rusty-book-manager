use super::user::{BookOwner, CheckoutUser};
use chrono::{DateTime, Utc};
use derive_new::new;
use garde::Validate;
use kernel::model::{
    book::{
        Book, BookListOptions, Checkout,
        event::{CreateBook, UpdateBook},
    },
    id::{BookId, CheckoutId, UserId},
    list::PaginatedList,
};
use serde::{Deserialize, Serialize};
use shared::error::AppError;

#[cfg(debug_assertions)]
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Validate)]
#[cfg_attr(debug_assertions, derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct CreateBookRequest {
    #[garde(length(min = 1))]
    pub title: String,
    #[garde(length(min = 1))]
    pub author: String,
    #[garde(length(min = 1))]
    pub isbn: String,
    #[garde(skip)]
    pub description: String,
}

impl TryFrom<CreateBookRequest> for CreateBook {
    type Error = AppError;

    fn try_from(value: CreateBookRequest) -> Result<Self, Self::Error> {
        let CreateBookRequest {
            title,
            author,
            isbn,
            description,
        } = value;
        Ok(CreateBook {
            title: title.parse()?,
            author: author.parse()?,
            isbn: isbn.parse()?,
            description: description.parse()?,
        })
    }
}

#[derive(Debug, Deserialize, Validate)]
#[cfg_attr(debug_assertions, derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct UpdateBookRequest {
    #[garde(length(min = 1))]
    pub title: String,
    #[garde(length(min = 1))]
    pub author: String,
    #[garde(length(min = 1))]
    pub isbn: String,
    #[garde(skip)]
    pub description: String,
}

#[derive(new)]
pub struct UpdateBookRequestWithIds(BookId, UserId, UpdateBookRequest);
impl TryFrom<UpdateBookRequestWithIds> for UpdateBook {
    type Error = AppError;

    fn try_from(value: UpdateBookRequestWithIds) -> Result<Self, Self::Error> {
        let UpdateBookRequestWithIds(
            book_id,
            user_id,
            UpdateBookRequest {
                title,
                author,
                isbn,
                description,
            },
        ) = value;
        Ok(UpdateBook {
            book_id,
            title: title.parse()?,
            author: author.parse()?,
            isbn: isbn.parse()?,
            description: description.parse()?,
            requested_user: user_id,
        })
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct BookListQuery {
    #[garde(range(min = 0))]
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[garde(range(min = 0))]
    #[serde(default)]
    pub offset: i64,
}

const DEFAULT_LIMIT: i64 = 20;
const fn default_limit() -> i64 {
    DEFAULT_LIMIT
}

impl From<BookListQuery> for BookListOptions {
    fn from(value: BookListQuery) -> Self {
        let BookListQuery { limit, offset } = value;
        Self { limit, offset }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(debug_assertions, derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct BookResponse {
    pub id: BookId,
    pub title: String,
    pub author: String,
    pub isbn: String,
    pub description: String,
    pub owner: BookOwner,
    pub checkout: Option<BookCheckoutResponse>,
}

impl From<Book> for BookResponse {
    fn from(value: Book) -> Self {
        let (id, title, author, isbn, description, owner, checkout) = value.into_parts();
        Self {
            id,
            title: title.into_inner(),
            author: author.into_inner(),
            isbn: isbn.into_inner(),
            description: description.into_inner(),
            owner: owner.into(),
            checkout: checkout.map(BookCheckoutResponse::from),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(debug_assertions, derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct PaginatedBookResponse {
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
    pub items: Vec<BookResponse>,
}

impl From<PaginatedList<Book>> for PaginatedBookResponse {
    fn from(value: PaginatedList<Book>) -> Self {
        let PaginatedList {
            total,
            limit,
            offset,
            items,
        } = value;
        Self {
            total,
            limit,
            offset,
            items: items.into_iter().map(BookResponse::from).collect(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(debug_assertions, derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct BookCheckoutResponse {
    pub id: CheckoutId,
    pub checked_out_by: CheckoutUser,
    pub checked_out_at: DateTime<Utc>,
}

impl From<Checkout> for BookCheckoutResponse {
    fn from(value: Checkout) -> Self {
        let (id, user, checked_out_at) = value.into_parts();
        Self {
            id,
            checked_out_by: user.into(),
            checked_out_at,
        }
    }
}
