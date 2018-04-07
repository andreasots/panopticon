use futures::IntoFuture;
use gotham::handler::HandlerFuture;
use gotham::middleware::Middleware;
use gotham::middleware::session::SessionData;
use gotham::state::State;
use hyper::Response;
use hyper::StatusCode;
use hyper::header::{ContentType, Headers, Host, Origin, Referer};
use hyper::Method;
use url::Url;
use hyper::mime::TEXT_HTML_UTF_8;
use gotham::http::response::create_response;
use tera::Context;
use gotham::handler::IntoHandlerError;

use ::Session;
use middleware::templates::Templates;
use middleware::request::Request;

#[derive(Clone, NewMiddleware)]
pub struct CsrfMiddleware {
}

impl CsrfMiddleware {
    pub fn new() -> Self {
        CsrfMiddleware {
        }
    }

    fn is_post(&self, state: &State) -> bool {
        *state.borrow::<Method>() == Method::Post
    }

    fn check_headers(&self, state: &State) -> bool {
        let headers = state.borrow::<Headers>();

        let host = match headers.get::<Host>() {
            Some(host) => host,
            None => return false,
        };

        if let Some(origin) = headers.get::<Origin>() {
            return origin.host() == Some(host);
        }

        if let Some(referrer) = headers.get::<Referer>() {
            return Url::parse(&referrer)
                .ok()
                .and_then(|url| {
                    let port = url.port();
                    url.host_str()
                        .map(|host| (host.to_string(), port))
                })
                .map(|(hostname, port)| &Host::new(hostname, port) == host)
                .unwrap_or(false);
        }

        false
    }

    fn check_token(&self, state: &State) -> bool {
        match (&state.borrow::<SessionData<Session>>().csrf_token, state.borrow::<Request>().get_first("csrf-token")) {
            (&Some(ref session_token), Some(form_token)) => session_token == form_token,
            _ => false,
        }
    }
}

impl Middleware for CsrfMiddleware {
    fn call<Chain>(self, state: State, chain: Chain) -> Box<HandlerFuture>
    where
        Chain: FnOnce(State) -> Box<HandlerFuture> + 'static,
    {
        if self.is_post(&state) && (!self.check_headers(&state) || !self.check_token(&state)) {
            let response = match state.borrow::<Templates>().render("csrf-validation-error.html", &Context::new()) {
                Ok(response) => response,
                Err(err) => return Box::new(Err((state, err.into_handler_error())).into_future()),
            };

            let response = create_response(
                &state,
                StatusCode::BadRequest,
                Some((response.into_bytes(), TEXT_HTML_UTF_8)),
            );

            return Box::new(Ok((state, response)).into_future());
        }

        chain(state)
    }
}
