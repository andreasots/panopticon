use diesel::prelude::*;
use futures::IntoFuture;
use gotham::middleware::Middleware;
use gotham::state::State;
use gotham::handler::{HandlerError, HandlerFuture, IntoHandlerError};
use gotham::middleware::session::SessionData;

use ::Session;
use models::User;
use middleware::diesel::DieselPool;

#[derive(NewMiddleware, Clone)]
pub struct UserMiddleware {
}

impl UserMiddleware {
    pub fn new() -> UserMiddleware {
        UserMiddleware {
        }
    }

    fn get_user(&self, state: &State) -> Result<Option<User>, HandlerError> {
        use schema::users::dsl::*;

        let user_id = match state.try_borrow::<SessionData<Session>>().and_then(|session| session.user_id) {
            Some(user_id) => user_id,
            None => return Ok(None),
        };

        let conn = state.borrow::<DieselPool>().get().map_err(IntoHandlerError::into_handler_error)?;

        users.filter(id.eq(user_id))
            .first::<User>(&conn)
            .optional()
            .map_err(IntoHandlerError::into_handler_error)
    }
}

impl Middleware for UserMiddleware {
    fn call<Chain>(self, mut state: State, chain: Chain) -> Box<HandlerFuture>
    where
        Chain: FnOnce(State) -> Box<HandlerFuture> + 'static,
    {
        let user = match self.get_user(&mut state) {
            Ok(user) => user,
            Err(err) => return Box::new(Err((state, err)).into_future()),
        };

        if let Some(user) = user {
            state.put(user);
        }

        chain(state)
    }
}
