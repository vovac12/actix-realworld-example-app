use actix_web::{web::Data, App};
use http::header::AUTHORIZATION;

use crate::{
    app::{
        routes,
        tests::{get_body, new_state},
    },
    db::tests::mocker::Mocker,
};
use actix_web::test;

#[actix_rt::test]
async fn test_misc() {
    let _ = actix::System::new("conduit");
    let mut app = test::init_service(
        App::new()
            .register_data(Data::new(new_state(Mocker::Ok)))
            .configure(routes),
    );
    let req = test::TestRequest::get().uri("/").to_request();
    let resp = test::call_service(&mut app, req);
    let body = get_body(resp.response());
    assert_eq!(body, "Hello world!");
}

#[actix_rt::test]
async fn test_users() {
    use crate::app::users;
    let _ = actix::System::new("conduit");
    let mut app = test::init_service(
        App::new()
            .register_data(Data::new(new_state(Mocker::Ok)))
            .configure(routes),
    );
    let req = test::TestRequest::get().uri("/api/users").to_request();
    let resp = test::call_service(&mut app, req);
    let body = get_body(resp.response());
    assert_eq!(body, "");

    let req = test::TestRequest::post()
        .uri("/api/users")
        .set_json(&users::In {
            user: users::RegisterUser::default(),
        })
        .to_request();
    let resp = test::call_service(&mut app, req);
    let body = get_body(resp.response());
    assert_eq!(
        body,
        serde_json::to_string(&users::UserResponse::default()).unwrap()
    );

    let req = test::TestRequest::post()
        .uri("/api/users/login")
        .set_json(&users::In {
            user: users::LoginUser::default(),
        })
        .to_request();
    let resp = test::call_service(&mut app, req);
    let body = get_body(resp.response());
    assert_eq!(
        body,
        serde_json::to_string(&users::UserResponse::default()).unwrap()
    );
}

#[actix_rt::test]
async fn test_user() {
    use crate::app::users;
    let _ = actix::System::new("conduit");
    let mut app = test::init_service(
        App::new()
            .register_data(Data::new(new_state(Mocker::Ok)))
            .configure(routes),
    );
    let req = test::TestRequest::get().uri("/api/user").to_request();
    let resp = test::call_service(&mut app, req);
    let body = get_body(resp.response());
    assert_eq!(
        body,
        r#"Unauthorized: {"error":"No authorization was provided"}"#
    );
    let req = test::TestRequest::get()
        .header(AUTHORIZATION, "Token j")
        .uri("/api/user")
        .to_request();
    let resp = test::call_service(&mut app, req);
    let body = get_body(resp.response());
    assert_eq!(
        body,
        serde_json::to_string(&users::UserResponse::default()).unwrap()
    );
}

#[actix_rt::test]
async fn test_profiles() {
    let _ = actix::System::new("conduit");
    let mut app = test::init_service(
        App::new()
            .register_data(Data::new(new_state(Mocker::Ok)))
            .configure(routes),
    );
    let req = test::TestRequest::get()
        .uri("/api/profiles/user")
        .to_request();
    let resp = test::call_service(&mut app, req);
    let body = get_body(resp.response());
    assert_eq!(
        body,
        r#"{"profile":{"username":"user","bio":null,"image":null,"following":true}}"#
    );

    let req = test::TestRequest::get()
        .uri("/api/profiles/user/follow")
        .to_request();
    let resp = test::call_service(&mut app, req);
    let body = get_body(resp.response());
    assert_eq!(body, "");
}

#[actix_rt::test]
async fn test_articles() {
    let _ = actix::System::new("conduit");
    let mut app = test::init_service(
        App::new()
            .register_data(Data::new(new_state(Mocker::Ok)))
            .configure(routes),
    );
    let req = test::TestRequest::get().uri("/api/articles").to_request();
    let resp = test::call_service(&mut app, req);
    let body = get_body(resp.response());
    assert_eq!(
        body,
        r#"{"articles":[{"slug":"slug","title":"title","description":"description","body":"body","tagList":["tag"],"createdAt":"1970-01-01T00:00:01.000Z","updatedAt":"1970-01-01T00:00:01.000Z","favorited":true,"favoritesCount":2,"author":{"username":"user","bio":null,"image":null,"following":true}}],"articlesCount":1}"#
    );

    let req = test::TestRequest::get()
        .uri("/api/articles/feed")
        .to_request();
    let resp = test::call_service(&mut app, req);
    let body = get_body(resp.response());
    assert_eq!(
        body,
        r#"Unauthorized: {"error":"No authorization was provided"}"#
    );

    let req = test::TestRequest::get()
        .uri("/api/articles/title")
        .to_request();
    let resp = test::call_service(&mut app, req);
    let body = get_body(resp.response());
    assert_eq!(
        body,
        r#"{"article":{"slug":"slug","title":"title","description":"description","body":"body","tagList":["tag"],"createdAt":"1970-01-01T00:00:01.000Z","updatedAt":"1970-01-01T00:00:01.000Z","favorited":true,"favoritesCount":2,"author":{"username":"user","bio":null,"image":null,"following":true}}}"#
    );
}

#[actix_rt::test]
async fn test_comments() {
    let _ = actix::System::new("conduit");
    let mut app = test::init_service(
        App::new()
            .register_data(Data::new(new_state(Mocker::Ok)))
            .configure(routes),
    );
    let req = test::TestRequest::get()
        .uri("/api/articles/title/comments")
        .to_request();
    let resp = test::call_service(&mut app, req);
    let body = get_body(resp.response());
    assert_eq!(
        body,
        r#"{"comments":[{"id":12,"createdAt":"1970-01-01T00:00:12.000Z","updatedAt":"1970-01-01T00:00:12.000Z","body":"Body","author":{"username":"User","bio":null,"image":null,"following":true}}]}"#
    );
}

#[actix_rt::test]
async fn test_tags() {
    let _ = actix::System::new("conduit");
    let mut app = test::init_service(
        App::new()
            .register_data(Data::new(new_state(Mocker::Ok)))
            .configure(routes),
    );
    let req = test::TestRequest::get().uri("/api/tags").to_request();
    let resp = test::call_service(&mut app, req);
    let body = get_body(resp.response());
    assert_eq!(body, r#"{"tags":["tag"]}"#);
}
