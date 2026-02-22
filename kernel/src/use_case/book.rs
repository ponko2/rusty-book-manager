use crate::{
    model::{
        book::{
            Book, BookListOptions,
            event::{CreateBook, DeleteBook, UpdateBook},
        },
        id::{BookId, UserId},
        list::PaginatedList,
    },
    unit_of_work::book::BookUnitOfWorkScope,
};
use async_trait::async_trait;
use shared::error::AppResult;
use std::sync::Arc;

#[mockall::automock]
#[async_trait]
pub trait BookUseCase: Send + Sync {
    async fn delete_book(&self, delete_book: DeleteBook) -> AppResult<()>;
    async fn register_book(&self, event: CreateBook, user_id: UserId) -> AppResult<()>;
    async fn show_book(&self, book_id: BookId) -> AppResult<Option<Book>>;
    async fn show_book_list(&self, options: BookListOptions) -> AppResult<PaginatedList<Book>>;
    async fn update_book(&self, update_book: UpdateBook) -> AppResult<()>;
}

pub struct BookUseCaseImpl {
    scope: Arc<dyn BookUnitOfWorkScope>,
}

impl BookUseCaseImpl {
    pub fn new(scope: Arc<dyn BookUnitOfWorkScope>) -> Self {
        Self { scope }
    }
}

#[async_trait]
impl BookUseCase for BookUseCaseImpl {
    async fn delete_book(&self, delete_book: DeleteBook) -> AppResult<()> {
        let uow = self.scope.begin().await?;
        uow.book_repository().delete(delete_book).await?;
        uow.commit().await
    }

    async fn register_book(&self, event: CreateBook, user_id: UserId) -> AppResult<()> {
        let uow = self.scope.begin().await?;
        uow.book_repository().create(event, user_id).await?;
        uow.commit().await
    }

    async fn show_book(&self, book_id: BookId) -> AppResult<Option<Book>> {
        let uow = self.scope.begin().await?;
        uow.book_repository().find_by_id(book_id).await
    }

    async fn show_book_list(&self, options: BookListOptions) -> AppResult<PaginatedList<Book>> {
        let uow = self.scope.begin().await?;
        uow.book_repository().find_all(options).await
    }

    async fn update_book(&self, update_book: UpdateBook) -> AppResult<()> {
        let uow = self.scope.begin().await?;
        uow.book_repository().update(update_book).await?;
        uow.commit().await
    }
}
