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
    use crate::app::tests::{get_body, new_state_value};

    #[test]
    fn test_get_some() {
        let mut sys = actix::System::new("conduit");
        let state = new_state_value(|| {
            Ok(TagsResponse {
                tags: vec!["Hello".to_string(), "world".to_string()],
            })
        });
        let resp = sys.block_on(get(Data::new(state))).unwrap();
        let body = get_body(&resp);
        assert_eq!(body, r#"{"tags":["Hello","world"]}"#);
    }

    #[test]
    fn test_get_empty() {
        let mut sys = actix::System::new("conduit");
        let state = new_state_value(|| Ok(TagsResponse { tags: vec![] }));
        let resp = sys.block_on(get(Data::new(state))).unwrap();
        let body = get_body(&resp);
        assert_eq!(body, r#"{"tags":[]}"#);
    }

    #[test]
    fn test_get_error() {
        let mut sys = actix::System::new("conduit");
        let state =
            new_state_value(|| -> Result<TagsResponse, _> { Err(Error::InternalServerError) });
        let resp = sys.block_on(get(Data::new(state))).unwrap();
        let body = get_body(&resp);
        assert_eq!(body, r#""Internal Server Error""#);
    }
}
