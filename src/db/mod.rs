mod articles;
mod auth;
mod comments;
mod profiles;
mod tags;
#[cfg(test)]
pub mod tests;
mod users;

use crate::prelude::*;
use actix::prelude::{Actor, SyncContext};
use diesel::{
    pg::PgConnection,
    r2d2::{self, ConnectionManager, Pool, PooledConnection},
};
#[cfg(test)]
use tests::mocker::Mocker;

pub type Conn = PgConnection;
pub type PgPool = Pool<ConnectionManager<Conn>>;
pub type PooledConn = PooledConnection<ConnectionManager<Conn>>;

pub struct RealDbExecutor(pub PgPool);

impl RealDbExecutor {
    pub fn new(pool: PgPool) -> Self {
        RealDbExecutor(pool)
    }
}

#[cfg(test)]
impl Into<Mocker<RealDbExecutor>> for RealDbExecutor {
    fn into(self) -> Mocker<RealDbExecutor> {
        Mocker::mock(Box::new(|x, _y| x))
    }
}

pub fn new_executor(pool: PgPool) -> DbExecutor {
    RealDbExecutor::new(pool).into()
}

#[cfg(not(test))]
pub type DbExecutor = RealDbExecutor;

#[cfg(test)]
pub type DbExecutor = Mocker<RealDbExecutor>;

impl Actor for RealDbExecutor {
    type Context = SyncContext<Self>;
}

pub fn new_pool<S: Into<String>>(database_url: S) -> Result<PgPool> {
    let manager = ConnectionManager::<Conn>::new(database_url.into());
    let pool = r2d2::Pool::builder().build(manager)?;
    Ok(pool)
}
