use tera::{Tera, Error};
use gotham::handler::HandlerFuture;
use gotham::middleware::Middleware;
use gotham::state::State;
use std::ops::Deref;
use std::sync::Arc;
use std::panic::AssertUnwindSafe;

use tera_helpers;

#[derive(Clone, NewMiddleware, StateData)]
pub struct Templates {
    tera: Arc<AssertUnwindSafe<Tera>>,
}

impl Templates {
    pub fn new(pattern: &str) -> Result<Templates, Error> {
        let mut tera = Tera::new(pattern)?;
        tera.register_global_function("webpacked", tera_helpers::make_webpacked());
        Ok(Templates {
            tera: Arc::new(AssertUnwindSafe(tera)),
        })
    }
}

impl Middleware for Templates {
    fn call<Chain>(self, mut state: State, chain: Chain) -> Box<HandlerFuture>
    where
        Chain: FnOnce(State) -> Box<HandlerFuture> + 'static,
    {
        state.put(self.clone());
        chain(state)
    }
}

impl Deref for Templates {
    type Target = Tera;

    fn deref(&self) -> &Self::Target {
        &self.tera.0
    }
}
