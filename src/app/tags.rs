use actix_web::{web::Data, HttpResponse, ResponseError};
use futures::Future;

use super::AppState;
use crate::prelude::*;

// Client Messages ↓

#[derive(Debug)]
pub struct GetTags {}

// JSON response objects ↓

#[derive(Serialize)]
pub struct TagsResponse {
    pub tags: Vec<String>,
}

// Route handlers ↓

pub fn get(state: Data<AppState>) -> impl Future<Item = HttpResponse, Error = Error> {
    state
        .db
        .send(GetTags {})
        .from_err()
        .and_then(|res| match res {
            Ok(res) => Ok(HttpResponse::Ok().json(res)),
            Err(e) => Ok(e.error_response()),
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::tests::mocker::OverwriteResult;
    use crate::{
        app::tests::{get_body, new_state},
        db::tests::mocker::Mocker,
    };

    impl OverwriteResult for GetTags {}

    impl Default for TagsResponse {
        fn default() -> Self {
            TagsResponse {
                tags: vec!["tag".to_string()],
            }
        }
    }

    #[test]
    fn test_get_some() {
        let mut sys = actix::System::new("conduit");
        let state = new_state(Mocker::Ok);
        let resp = sys.block_on(get(Data::new(state))).unwrap();
        let body = get_body(&resp);
        assert_eq!(body, r#"{"tags":["tag"]}"#);
    }

    #[test]
    fn test_get_empty() {
        let mut sys = actix::System::new("conduit");
        let state = new_state(Mocker::NotFound);
        let resp = sys.block_on(get(Data::new(state))).unwrap();
        let body = get_body(&resp);
        assert_eq!(body, r#""NotFound""#);
    }

    #[test]
    fn test_get_error() {
        let mut sys = actix::System::new("conduit");
        let state = new_state(Mocker::InternalError);
        let resp = sys.block_on(get(Data::new(state))).unwrap();
        let body = get_body(&resp);
        assert_eq!(body, r#""Internal Server Error""#);
    }
}
