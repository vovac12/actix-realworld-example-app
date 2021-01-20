pub mod comments;

use actix_http::error::ResponseError;
use actix_web::{web::Data, web::Json, web::Path, web::Query, HttpRequest, HttpResponse};
use futures::{future::result, Future};
use validator::Validate;

use super::AppState;
use crate::app::profiles::ProfileResponseInner;
use crate::prelude::*;
use crate::utils::{
    auth::{authenticate, Auth},
    CustomDateTime,
};

#[derive(Debug, Deserialize)]
pub struct In<T> {
    article: T,
}

// Extractors ↓

#[derive(Debug, Deserialize)]
pub struct ArticlePath {
    pub slug: String,
}

#[derive(Debug, Deserialize)]
pub struct ArticlesParams {
    pub tag: Option<String>,
    pub author: Option<String>,
    pub favorited: Option<String>,
    pub limit: Option<usize>,  // <- if not set, is 20
    pub offset: Option<usize>, // <- if not set, is 0
}

#[derive(Debug, Deserialize)]
pub struct FeedParams {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

// Client Messages ↓

#[derive(Debug, Validate, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateArticle {
    #[validate(length(min = "1", message = "fails validation - cannot be empty"))]
    pub title: String,
    #[validate(length(min = "1", message = "fails validation - cannot be empty"))]
    pub description: String,
    #[validate(length(min = "1", message = "fails validation - cannot be empty"))]
    pub body: String,
    pub tag_list: Vec<String>,
}

#[derive(Debug)]
pub struct CreateArticleOuter {
    pub auth: Auth,
    pub article: CreateArticle,
}

#[derive(Debug)]
pub struct GetArticle {
    pub auth: Option<Auth>,
    pub slug: String,
}

#[derive(Debug, Validate, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateArticle {
    #[validate(length(min = "1", message = "fails validation - cannot be empty"))]
    pub title: Option<String>,
    #[validate(length(min = "1", message = "fails validation - cannot be empty"))]
    pub description: Option<String>,
    #[validate(length(min = "1", message = "fails validation - cannot be empty"))]
    pub body: Option<String>,
    #[validate(length(min = "1", message = "fails validation - cannot be empty"))]
    pub tag_list: Option<Vec<String>>,
}

#[derive(Debug)]
pub struct UpdateArticleOuter {
    pub auth: Auth,
    pub slug: String,
    pub article: UpdateArticle,
}

#[derive(Debug)]
pub struct DeleteArticle {
    pub auth: Auth,
    pub slug: String,
}

#[derive(Debug)]
pub struct FavoriteArticle {
    pub auth: Auth,
    pub slug: String,
}

#[derive(Debug)]
pub struct UnfavoriteArticle {
    pub auth: Auth,
    pub slug: String,
}

#[derive(Debug)]
pub struct GetArticles {
    pub auth: Option<Auth>,
    pub params: ArticlesParams,
}

#[derive(Debug)]
pub struct GetFeed {
    pub auth: Auth,
    pub params: FeedParams,
}

// JSON response objects ↓

#[derive(Debug, Serialize)]
pub struct ArticleResponse {
    pub article: ArticleResponseInner,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArticleResponseInner {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub body: String,
    pub tag_list: Vec<String>,
    pub created_at: CustomDateTime,
    pub updated_at: CustomDateTime,
    pub favorited: bool,
    pub favorites_count: usize,
    pub author: ProfileResponseInner,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArticleListResponse {
    pub articles: Vec<ArticleResponseInner>,
    pub articles_count: usize,
}

// Route handlers ↓

pub fn create(
    state: Data<AppState>,
    (form, req): (Json<In<CreateArticle>>, HttpRequest),
) -> impl Future<Item = HttpResponse, Error = Error> {
    let article = form.into_inner().article;
    let db = state.db.clone();

    result(article.validate())
        .from_err()
        .and_then(move |_| authenticate(&state, &req))
        .and_then(move |auth| db.send(CreateArticleOuter { auth, article }).from_err())
        .and_then(|res| match res {
            Ok(res) => Ok(HttpResponse::Ok().json(res)),
            Err(e) => Ok(e.error_response()),
        })
}

pub fn get(
    state: Data<AppState>,
    (path, req): (Path<ArticlePath>, HttpRequest),
) -> impl Future<Item = HttpResponse, Error = Error> {
    let db = state.db.clone();

    authenticate(&state, &req)
        .then(move |auth| {
            db.send(GetArticle {
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

pub fn update(
    state: Data<AppState>,
    (path, form, req): (Path<ArticlePath>, Json<In<UpdateArticle>>, HttpRequest),
) -> impl Future<Item = HttpResponse, Error = Error> {
    let article = form.into_inner().article;

    let db = state.db.clone();

    result(article.validate())
        .from_err()
        .and_then(move |_| authenticate(&state, &req))
        .and_then(move |auth| {
            db.send(UpdateArticleOuter {
                auth,
                slug: path.slug.to_owned(),
                article,
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
    (path, req): (Path<ArticlePath>, HttpRequest),
) -> impl Future<Item = HttpResponse, Error = Error> {
    authenticate(&state, &req)
        .and_then(move |auth| {
            state
                .db
                .send(DeleteArticle {
                    auth,
                    slug: path.slug.to_owned(),
                })
                .from_err()
        })
        .and_then(|res| match res {
            Ok(_) => Ok(HttpResponse::Ok().finish()),
            Err(e) => Ok(e.error_response()),
        })
}

pub fn favorite(
    state: Data<AppState>,
    (path, req): (Path<ArticlePath>, HttpRequest),
) -> impl Future<Item = HttpResponse, Error = Error> {
    authenticate(&state, &req)
        .and_then(move |auth| {
            state
                .db
                .send(FavoriteArticle {
                    auth,
                    slug: path.slug.to_owned(),
                })
                .from_err()
        })
        .and_then(|res| match res {
            Ok(res) => Ok(HttpResponse::Ok().json(res)),
            Err(e) => Ok(e.error_response()),
        })
}

pub fn unfavorite(
    state: Data<AppState>,
    (path, req): (Path<ArticlePath>, HttpRequest),
) -> impl Future<Item = HttpResponse, Error = Error> {
    authenticate(&state, &req)
        .and_then(move |auth| {
            state
                .db
                .send(UnfavoriteArticle {
                    auth,
                    slug: path.slug.to_owned(),
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
    (req, params): (HttpRequest, Query<ArticlesParams>),
) -> impl Future<Item = HttpResponse, Error = Error> {
    let db = state.db.clone();

    authenticate(&state, &req)
        .then(move |auth| {
            db.send(GetArticles {
                auth: auth.ok(),
                params: params.into_inner(),
            })
            .from_err()
        })
        .and_then(|res| match res {
            Ok(res) => Ok(HttpResponse::Ok().json(res)),
            Err(e) => Ok(e.error_response()),
        })
}

pub fn feed(
    state: Data<AppState>,
    (req, params): (HttpRequest, Query<FeedParams>),
) -> impl Future<Item = HttpResponse, Error = Error> {
    let db = state.db.clone();

    authenticate(&state, &req)
        .and_then(move |auth| {
            db.send(GetFeed {
                auth,
                params: params.into_inner(),
            })
            .from_err()
        })
        .and_then(|res| match res {
            Ok(res) => Ok(HttpResponse::Ok().json(res)),
            Err(e) => Ok(e.error_response()),
        })
}

#[cfg(test)]
mod tests {
    use actix::SystemRunner;
    use chrono::NaiveDateTime;
    use http::header::AUTHORIZATION;
    use serde::Serialize;

    use crate::db::tests::mocker::OverwriteResult;
    use crate::{app::tests::*, db::tests::mocker::Mocker};

    use super::*;

    impl OverwriteResult for GetFeed {}
    impl OverwriteResult for GetArticle {}
    impl OverwriteResult for GetArticles {}
    impl OverwriteResult for UnfavoriteArticle {}
    impl OverwriteResult for FavoriteArticle {}
    impl OverwriteResult for DeleteArticle {}
    impl OverwriteResult for UpdateArticleOuter {}
    impl OverwriteResult for CreateArticleOuter {}

    impl Default for CreateArticle {
        fn default() -> Self {
            CreateArticle {
                title: "title".to_string(),
                description: "description".to_string(),
                body: "body".to_string(),
                tag_list: vec!["tag".to_string()],
            }
        }
    }

    impl Default for ArticleResponseInner {
        fn default() -> Self {
            ArticleResponseInner {
                slug: "slug".to_string(),
                title: "title".to_string(),
                description: "description".to_string(),
                body: "body".to_string(),
                tag_list: vec!["tag".to_string()],
                created_at: CustomDateTime(NaiveDateTime::from_timestamp(1, 2)),
                updated_at: CustomDateTime(NaiveDateTime::from_timestamp(1, 2)),
                favorited: true,
                favorites_count: 2,
                author: ProfileResponseInner::default(),
            }
        }
    }

    impl Default for ArticleResponse {
        fn default() -> Self {
            ArticleResponse {
                article: ArticleResponseInner::default(),
            }
        }
    }

    impl Default for ArticleListResponse {
        fn default() -> Self {
            ArticleListResponse {
                articles: vec![ArticleResponseInner::default()],
                articles_count: 1,
            }
        }
    }

    impl Default for GetArticle {
        fn default() -> Self {
            GetArticle {
                auth: Some(Auth::default()),
                slug: "slug".to_string(),
            }
        }
    }

    impl Default for UpdateArticle {
        fn default() -> Self {
            UpdateArticle {
                title: Some("title".to_string()),
                description: Some("description".to_string()),
                body: Some("body".to_string()),
                tag_list: Some(vec!["tag".to_string()]),
            }
        }
    }

    impl Default for UpdateArticleOuter {
        fn default() -> Self {
            UpdateArticleOuter {
                auth: Auth::default(),
                slug: "slug".to_string(),
                article: UpdateArticle::default(),
            }
        }
    }

    #[test]
    fn test_create_some() {
        let mut sys = actix::System::new("conduit");
        let state = new_state(Mocker::Ok);
        let req =
            actix_web::test::TestRequest::with_header(AUTHORIZATION, "Token sj").to_http_request();
        let resp = sys
            .block_on(create(
                Data::new(state),
                (
                    Json(In {
                        article: CreateArticle::default(),
                    }),
                    HttpRequest::from(req),
                ),
            ))
            .unwrap();
        let body = get_body(&resp);
        assert_eq!(resp.status(), 200);
        assert_eq!(
            body,
            serde_json::to_string(&ArticleResponse::default()).unwrap()
        );
    }

    #[test]
    fn test_create_unauthorized() {
        let mut sys = actix::System::new("conduit");
        let state = new_state(Mocker::Ok);
        let req = actix_web::test::TestRequest::default().to_http_request();
        let resp = sys
            .block_on(create(
                Data::new(state),
                (
                    Json(In {
                        article: CreateArticle::default(),
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

    fn abstract_test<
        A: 'static + Default + Serialize,
        B: 'static + Default,
        F: Fn(AppState, &mut SystemRunner, HttpRequest) -> HttpResponse,
    >(
        f: F,
        m: Mocker,
        auth: bool,
        expected: &str,
    ) {
        let mut sys = actix::System::new("conduit");
        let state = new_state(m);
        let req = if auth {
            actix_web::test::TestRequest::with_header(AUTHORIZATION, "Token sj").to_http_request()
        } else {
            actix_web::test::TestRequest::default().to_http_request()
        };
        let resp = f(state, &mut sys, req);
        let body = get_body(&resp);
        assert_eq!(resp.status(), 200);
        assert_eq!(body, expected);
    }

    #[test]
    fn test_get_some() {
        abstract_test::<ArticleResponse, GetArticle, _>(
            |a, s, r| {
                s.block_on(get(
                    Data::new(a),
                    (
                        Path::from(ArticlePath {
                            slug: "slug".to_string(),
                        }),
                        r,
                    ),
                ))
                .unwrap()
            },
            Mocker::Ok,
            true,
            &serde_json::to_string(&ArticleResponse::default()).unwrap(),
        )
    }

    #[test]
    fn test_get_unauthorized() {
        abstract_test::<ArticleResponse, GetArticle, _>(
            |a, s, r| {
                s.block_on(get(
                    Data::new(a),
                    (
                        Path::from(ArticlePath {
                            slug: "slug".to_string(),
                        }),
                        r,
                    ),
                ))
                .unwrap()
            },
            Mocker::Ok,
            false,
            &serde_json::to_string(&ArticleResponse::default()).unwrap(),
        )
    }
}
