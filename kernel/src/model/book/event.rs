use crate::model::{
    id::{BookId, UserId},
    value::{BookAuthor, BookDescription, BookIsbn, BookTitle},
};

#[derive(Debug)]
pub struct CreateBook {
    pub title: BookTitle,
    pub author: BookAuthor,
    pub isbn: BookIsbn,
    pub description: BookDescription,
}

#[derive(Debug)]
pub struct UpdateBook {
    pub book_id: BookId,
    pub title: BookTitle,
    pub author: BookAuthor,
    pub isbn: BookIsbn,
    pub description: BookDescription,
    pub requested_user: UserId,
}

#[derive(Debug)]
pub struct DeleteBook {
    pub book_id: BookId,
    pub requested_user: UserId,
}
