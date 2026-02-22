use crate::{
    deserialize_json,
    helper::{TestRequestExt, fixture, make_router, v1},
};
use api::model::book::PaginatedBookResponse;
use axum::{body::Body, http::Request};
use kernel::{
    model::{
        book::Book,
        id::{BookId, UserId},
        list::PaginatedList,
        user::BookOwner,
    },
    use_case::book::MockBookUseCase,
};
use rstest::rstest;
use std::sync::Arc;
use tower::ServiceExt;

#[rstest]
#[case("/books", 20, 0)]
#[case("/books?limit=50", 50, 0)]
#[case("/books?limit=50&offset=20", 50, 20)]
#[case("/books?offset=20", 20, 20)]
#[tokio::test]
async fn show_book_list_with_query_200(
    mut fixture: registry::MockAppRegistryExt,
    #[case] path: &str,
    #[case] expected_limit: i64,
    #[case] expected_offset: i64,
) -> anyhow::Result<()> {
    let book_id = BookId::new();

    fixture.expect_book_use_case().returning(move || {
        use kernel::use_case::book::MockBookUseCase;

        let mut mock = MockBookUseCase::new();
        mock.expect_show_book_list().returning(move |opt| {
            let items = vec![Book::new(
                book_id,
                "RustによるWebアプリケーション開発".parse().unwrap(),
                "Yuki Toyoda".parse().unwrap(),
                "978-4-00-000000-0".parse().unwrap(),
                "RustによるWebアプリケーション開発".parse().unwrap(),
                BookOwner::new(UserId::new(), "Yuki Toyoda".parse().unwrap()),
                None,
            )];
            Ok(PaginatedList {
                total: 1,
                limit: opt.limit,
                offset: opt.offset,
                items,
            })
        });
        Arc::new(mock)
    });

    let app: axum::Router = make_router(fixture);

    let req = Request::get(v1(path)).bearer().body(Body::empty())?;
    let resp = app.oneshot(req).await?;
    assert_eq!(resp.status(), axum::http::StatusCode::OK);

    let result = deserialize_json!(resp, PaginatedBookResponse);
    assert_eq!(result.limit, expected_limit);
    assert_eq!(result.offset, expected_offset);

    Ok(())
}

#[rstest]
#[case("/books?limit=-1")]
#[case("/books?offset=aaa")]
#[tokio::test]
async fn show_book_list_with_query_400(
    mut fixture: registry::MockAppRegistryExt,
    #[case] path: &str,
) -> anyhow::Result<()> {
    let book_id = BookId::new();

    fixture.expect_book_use_case().returning(move || {
        let mut mock = MockBookUseCase::new();
        mock.expect_show_book_list().returning(move |opt| {
            let items = vec![Book::new(
                book_id,
                "RustによるWebアプリケーション開発".parse().unwrap(),
                "Yuki Toyoda".parse().unwrap(),
                "978-4-00-000000-0".parse().unwrap(),
                "RustによるWebアプリケーション開発".parse().unwrap(),
                BookOwner::new(UserId::new(), "Yuki Toyoda".parse().unwrap()),
                None,
            )];
            Ok(PaginatedList {
                total: 1,
                limit: opt.limit,
                offset: opt.offset,
                items,
            })
        });
        Arc::new(mock)
    });

    let app: axum::Router = make_router(fixture);

    let req = Request::get(v1(path)).bearer().body(Body::empty())?;
    let resp = app.oneshot(req).await?;
    assert_eq!(resp.status(), axum::http::StatusCode::BAD_REQUEST);

    Ok(())
}
