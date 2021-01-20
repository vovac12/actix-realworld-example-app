use actix_http::error::ResponseError;
use actix_web::{web::Data, web::Json, web::Path, HttpRequest, HttpResponse};
use futures::{future::result, Future};
use validator::Validate;

use super::super::AppState;
use crate::app::profiles::ProfileResponseInner;
use crate::prelude::*;
use crate::utils::{
    auth::{authenticate, Auth},
    CustomDateTime,
};

#[derive(Debug, Deserialize)]
pub struct In<T> {
    comment: T,
}

// Extractors ↓

use super::ArticlePath;

#[derive(Debug, Deserialize)]
pub struct ArticleCommentPath {
    slug: String,
    comment_id: i32,
}

// Client Messages ↓

#[derive(Debug, Validate, Deserialize)]
pub struct AddComment {
    #[validate(length(min = "1", message = "fails validation - cannot be empty"))]
    pub body: String,
}

#[derive(Debug)]
pub struct AddCommentOuter {
    pub auth: Auth,
    pub slug: String,
    pub comment: AddComment,
}

#[derive(Debug)]
pub struct GetComments {
    pub auth: Option<Auth>,
    pub slug: String,
}

#[derive(Debug)]
pub struct DeleteComment {
    pub auth: Auth,
    pub slug: String,
    pub comment_id: i32,
}

// JSON response objects ↓

#[derive(Debug, Serialize)]
pub struct CommentResponse {
    pub comment: CommentResponseInner,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentResponseInner {
    pub id: i32,
    pub created_at: CustomDateTime,
    pub updated_at: CustomDateTime,
    pub body: String,
    pub author: ProfileResponseInner,
}

#[derive(Debug, Serialize)]
pub struct CommentListResponse {
    pub comments: Vec<CommentResponseInner>,
}

// Route handlers ↓

pub fn add(
    state: Data<AppState>,
    (path, form, req): (Path<ArticlePath>, Json<In<AddComment>>, HttpRequest),
) -> impl Future<Item = HttpResponse, Error = Error> {
    let comment = form.into_inner().comment;

    let db = state.db.clone();

    result(comment.validate())
        .from_err()
        .and_then(move |_| authenticate(&state, &req))
        .and_then(move |auth| {
            db.send(AddCommentOuter {
                auth,
                slug: path.slug.to_owned(),
                comment,
            })
            .from_err()
        })
        .and_then(|res| match res {
            Ok(res) => Ok(HttpResponse::Ok().json(res)),
            Err(e) => Ok(e.error_response()),
        })
}

pub fn list(
    state: Data<AppState>,
    (path, req): (Path<ArticlePath>, HttpRequest),
) -> impl Future<Item = HttpResponse, Error = Error> {
    let db = state.db.clone();

    authenticate(&state, &req)
        .then(move |auth| {
            db.send(GetComments {
                auth: auth.ok(),
                slug: path.slug.to_owned(),
            })
            .from_err()
        })
        .and_then(|res| match res {
            Ok(res) => Ok(HttpResponse::Ok().json(res)),
            Err(e) => Ok(e.error_response()),
        })
}

pub fn delete(
    state: Data<AppState>,
    (path, req): (Path<ArticleCommentPath>, HttpRequest),
) -> impl Future<Item = HttpResponse, Error = Error> {
    let db = state.db.clone();

    authenticate(&state, &req)
        .and_then(move |auth| {
            db.send(DeleteComment {
                auth,
                slug: path.slug.to_owned(),
                comment_id: path.comment_id.to_owned(),
            })
            .from_err()
        })
        .and_then(|res| match res {
            Ok(_) => Ok(HttpResponse::Ok().finish()),
            Err(e) => Ok(e.error_response()),
        })
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDateTime;
    use http::header::AUTHORIZATION;

    use super::*;
    use crate::{
        app::tests::{get_body, new_state},
        db::tests::mocker::{Mocker, OverwriteResult},
    };

    impl OverwriteResult for GetComments {}
    impl OverwriteResult for DeleteComment {}
    impl OverwriteResult for AddCommentOuter {}

    impl Default for AddComment {
        fn default() -> Self {
            AddComment {
                body: "Body".to_string(),
            }
        }
    }

    impl Default for AddCommentOuter {
        fn default() -> Self {
            AddCommentOuter {
                comment: AddComment::default(),
                auth: Auth::default(),
                slug: "slug".to_string(),
            }
        }
    }

    impl Default for CommentResponseInner {
        fn default() -> Self {
            CommentResponseInner {
                id: 12,
                created_at: CustomDateTime(NaiveDateTime::from_timestamp(12, 12)),
                updated_at: CustomDateTime(NaiveDateTime::from_timestamp(12, 2)),
                body: "Body".to_string(),
                author: ProfileResponseInner {
                    username: "User".to_string(),
                    bio: None,
                    image: None,
                    following: true,
                },
            }
        }
    }

    impl Default for CommentResponse {
        fn default() -> Self {
            CommentResponse {
                comment: CommentResponseInner::default(),
            }
        }
    }

    impl Default for CommentListResponse {
        fn default() -> Self {
            CommentListResponse {
                comments: vec![CommentResponseInner::default()],
            }
        }
    }

    #[test]
    fn test_delete_some() {
        let mut sys = actix::System::new("conduit");
        let state = new_state(Mocker::Ok);
        let req =
            actix_web::test::TestRequest::with_header(AUTHORIZATION, "Token sj").to_http_request();
        let resp = sys
            .block_on(delete(
                Data::new(state),
                (
                    Path::from(ArticleCommentPath {
                        slug: "a".to_string(),
                        comment_id: 3,
                    }),
                    HttpRequest::from(req),
                ),
            ))
            .unwrap();
        let body = get_body(&resp);
        assert_eq!(body, r#""#);
    }

    #[test]
    fn test_delete_unauthorized() {
        let mut sys = actix::System::new("conduit");
        let state = new_state(Mocker::Ok);
        let req = actix_web::test::TestRequest::default().to_http_request();
        let resp = sys
            .block_on(delete(
                Data::new(state),
                (
                    Path::from(ArticleCommentPath {
                        slug: "a".to_string(),
                        comment_id: 3,
                    }),
                    HttpRequest::from(req),
                ),
            ))
            .unwrap_err()
            .error_response();
        let body = get_body(&resp);
        assert_eq!(body, r#"{"error":"No authorization was provided"}"#);
    }

    #[test]
    fn test_delete_not_exists() {
        let mut sys = actix::System::new("conduit");
        let state = new_state(Mocker::NotFound);
        let req = actix_web::test::TestRequest::default()
            .header(AUTHORIZATION, "Token d")
            .to_http_request();
        let resp = sys
            .block_on(delete(
                Data::new(state),
                (
                    Path::from(ArticleCommentPath {
                        slug: "a".to_string(),
                        comment_id: 3,
                    }),
                    HttpRequest::from(req),
                ),
            ))
            .unwrap_err()
            .error_response();
        let body = get_body(&resp);
        assert_eq!(body, r#""NotFound""#);
        assert_eq!(resp.status(), 404);
    }

    #[test]
    fn test_list_not_found() {
        let mut sys = actix::System::new("conduit");
        let state = new_state(Mocker::NotFound);
        let req =
            actix_web::test::TestRequest::with_header(AUTHORIZATION, "Token sj").to_http_request();
        let resp = sys
            .block_on(list(
                Data::new(state),
                (
                    Path::from(ArticlePath {
                        slug: "a".to_string(),
                    }),
                    HttpRequest::from(req),
                ),
            ))
            .unwrap();
        let body = get_body(&resp);
        assert_eq!(resp.status(), 404);
        assert_eq!(body, r#""NotFound""#);
    }

    #[test]
    fn test_list_some() {
        let mut sys = actix::System::new("conduit");
        let state = new_state(Mocker::Ok);
        let req =
            actix_web::test::TestRequest::with_header(AUTHORIZATION, "Token sj").to_http_request();
        let resp = sys
            .block_on(list(
                Data::new(state),
                (
                    Path::from(ArticlePath {
                        slug: "a".to_string(),
                    }),
                    HttpRequest::from(req),
                ),
            ))
            .unwrap();
        let body = get_body(&resp);
        assert_eq!(resp.status(), 200);
        assert_eq!(
            body,
            r#"{"comments":[{"id":12,"createdAt":"1970-01-01T00:00:12.000Z","updatedAt":"1970-01-01T00:00:12.000Z","body":"Body","author":{"username":"User","bio":null,"image":null,"following":true}}]}"#
        );
    }

    #[test]
    fn test_list_authorized() {
        let mut sys = actix::System::new("conduit");
        let state = new_state(Mocker::Ok);
        let req =
            actix_web::test::TestRequest::with_header(AUTHORIZATION, "Token sj").to_http_request();
        let resp = sys
            .block_on(list(
                Data::new(state),
                (
                    Path::from(ArticlePath {
                        slug: "a".to_string(),
                    }),
                    HttpRequest::from(req),
                ),
            ))
            .unwrap();
        let body = get_body(&resp);
        assert_eq!(resp.status(), 200);
        assert_eq!(
            body,
            r#"{"comments":[{"id":12,"createdAt":"1970-01-01T00:00:12.000Z","updatedAt":"1970-01-01T00:00:12.000Z","body":"Body","author":{"username":"User","bio":null,"image":null,"following":true}}]}"#
        );
    }

    #[test]
    fn test_add_unauthorized() {
        let mut sys = actix::System::new("conduit");
        let state = new_state(Mocker::Ok);
        let req = actix_web::test::TestRequest::default().to_http_request();
        let resp = sys
            .block_on(add(
                Data::new(state),
                (
                    Path::from(ArticlePath {
                        slug: "a".to_string(),
                    }),
                    Json(In {
                        comment: AddComment::default(),
                    }),
                    HttpRequest::from(req),
                ),
            ))
            .unwrap_err()
            .error_response();
        let body = get_body(&resp);
        assert_eq!(resp.status(), 401);
        assert_eq!(body, r#"{"error":"No authorization was provided"}"#);
    }

    #[test]
    fn test_add_authorized() {
        let mut sys = actix::System::new("conduit");
        let state = new_state(Mocker::Ok);
        let req =
            actix_web::test::TestRequest::with_header(AUTHORIZATION, "Token sj").to_http_request();
        let resp = sys
            .block_on(add(
                Data::new(state),
                (
                    Path::from(ArticlePath {
                        slug: "a".to_string(),
                    }),
                    Json(In {
                        comment: AddComment::default(),
                    }),
                    HttpRequest::from(req),
                ),
            ))
            .unwrap();
        let body = get_body(&resp);
        assert_eq!(resp.status(), 200);
        assert_eq!(
            body,
            r#"{"comment":{"id":12,"createdAt":"1970-01-01T00:00:12.000Z","updatedAt":"1970-01-01T00:00:12.000Z","body":"Body","author":{"username":"User","bio":null,"image":null,"following":true}}}"#
        );
    }
}
