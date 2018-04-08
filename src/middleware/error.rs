use gotham::handler::HandlerFuture;
use gotham::middleware::Middleware;
use gotham::state::State;
use slog_scope::logger;
use futures::Future;
use std::error::Error;

#[derive(Clone, NewMiddleware, StateData)]
pub struct ErrorMiddleware {
}

impl ErrorMiddleware {
    pub fn new() -> ErrorMiddleware {
        ErrorMiddleware {
        }
    }
}

impl Middleware for ErrorMiddleware {
    fn call<Chain>(self, state: State, chain: Chain) -> Box<HandlerFuture>
    where
        Chain: FnOnce(State) -> Box<HandlerFuture> + 'static,
    {
        Box::new(chain(state)
            .map_err(|(state, err)| {
                error!(logger(), "error while processing request"; "error" => ?err.cause());

                (state, err)
            })
        )
    }
}
