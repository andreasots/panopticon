use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use gotham::handler::HandlerFuture;
use gotham::middleware::Middleware;
use gotham::state::State;
use std::ops::Deref;
use std::panic::AssertUnwindSafe;

#[derive(StateData)]
pub struct DieselPool {
    pool: Pool<ConnectionManager<PgConnection>>,
}

impl Deref for DieselPool {
    type Target = Pool<ConnectionManager<PgConnection>>;

    fn deref(&self) -> &Self::Target {
        &self.pool
    }
}

#[derive(NewMiddleware)]
pub struct DieselMiddleware {
    pool: AssertUnwindSafe<Pool<ConnectionManager<PgConnection>>>,
}

impl DieselMiddleware {
    pub fn new(pool: Pool<ConnectionManager<PgConnection>>) -> Self {
        DieselMiddleware {
            pool: AssertUnwindSafe(pool),
        }
    }
}

impl Clone for DieselMiddleware {
    fn clone(&self) -> Self {
        DieselMiddleware {
            pool: AssertUnwindSafe(self.pool.0.clone()),
        }
    }
}

impl Middleware for DieselMiddleware {
    fn call<Chain>(self, mut state: State, chain: Chain) -> Box<HandlerFuture>
    where
        Chain: FnOnce(State) -> Box<HandlerFuture> + 'static,
    {
        state.put(DieselPool {
            pool: self.pool.0.clone(),
        });
        chain(state)
    }
}
