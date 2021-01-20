use crate::prelude::{Error, Result};
use actix::prelude::*;
use dev::MessageResponse;

#[derive(Debug, Clone)]
pub enum Mocker {
    Ok,
    NotFound,
    InternalError,
    Unauthorized,
}

impl Default for Mocker {
    fn default() -> Self {
        Mocker::Ok
    }
}

impl Actor for Mocker {
    type Context = SyncContext<Self>;
}

pub trait OverwriteResult {
    fn overwrite_result(&self, m: &Mocker) -> Option<Mocker> {
        Some(m.clone())
    }
}

pub trait DefaultValue {
    fn default_value() -> Self;
    fn none_value() -> Self;
}

impl<T: Default> From<Mocker> for Result<T> {
    fn from(val: Mocker) -> Self {
        match val {
            Mocker::Ok => Ok(T::default()),
            Mocker::NotFound => Err(Error::NotFound(json!("NotFound"))),
            Mocker::Unauthorized => Err(Error::Unauthorized(json!("Unauthorized"))),
            Mocker::InternalError => Err(Error::InternalServerError),
        }
    }
}

impl<M: 'static> Handler<M> for Mocker
where
    M: Message + OverwriteResult,
    <M as Message>::Result: MessageResponse<Mocker, M> + From<Mocker>,
{
    type Result = M::Result;
    fn handle(&mut self, msg: M, _: &mut Self::Context) -> M::Result {
        match msg.overwrite_result(self) {
            None => self.clone(),
            Some(x) => x,
        }
        .into()
    }
}
