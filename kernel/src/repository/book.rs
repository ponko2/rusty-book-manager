use crate::model::{
    book::{
        Book, BookListOptions,
        event::{CreateBook, DeleteBook, UpdateBook},
    },
    id::{BookId, UserId},
    list::PaginatedList,
};
use async_trait::async_trait;
use shared::error::AppResult;

#[async_trait]
pub trait BookRepository: Send + Sync {
    async fn create(&self, event: CreateBook, user_id: UserId) -> AppResult<()>;
    async fn find_all(&self, options: BookListOptions) -> AppResult<PaginatedList<Book>>;
    async fn find_by_id(&self, book_id: BookId) -> AppResult<Option<Book>>;
    async fn update(&self, event: UpdateBook) -> AppResult<()>;
    async fn delete(&self, event: DeleteBook) -> AppResult<()>;
}
