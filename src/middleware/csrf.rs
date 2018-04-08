use futures::IntoFuture;
use gotham::handler::HandlerFuture;
use gotham::middleware::Middleware;
use gotham::middleware::session::SessionData;
use gotham::state::State;
use hyper::StatusCode;
use hyper::header::{Headers, Host, Origin, Referer};
use hyper::Method;
use url::Url;
use hyper::mime::TEXT_HTML_UTF_8;
use gotham::http::response::create_response;
use gotham::handler::IntoHandlerError;
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;
use slog_scope::logger;

use ::Session;
use ::Context;
use middleware::templates::Templates;
use middleware::request::Request;

#[derive(Clone, NewMiddleware, StateData)]
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
            None => {
                info!(logger(), "CSRF validation failed, Host header missing");
                return false;
            }
        };

        if let Some(origin) = headers.get::<Origin>() {
            if origin.host() != Some(host) {
                info!(logger(), "CSRF validation failed, Origin doesn't match Host"; "Origin" => ?origin.host(), "Host" => ?host);
                return false;
            } else {
                return true;
            }
        }

        if let Some(referrer) = headers.get::<Referer>() {
            let matches = Url::parse(&referrer)
                .ok()
                .and_then(|url| {
                    let port = url.port();
                    url.host_str()
                        .map(|host| (host.to_string(), port))
                })
                .map(|(hostname, port)| &Host::new(hostname, port) == host)
                .unwrap_or(false);
            
            if ! matches {
                info!(logger(), "CSRF validation failed, Referer doesn't match Host"; "Referer" => ?referrer, "Host" => ?host);
            }

            return matches;
        }

        info!(logger(), "CSRF validation failed, both Origin and Referer are missing");

        false
    }

    fn check_token(&self, state: &State) -> bool {
        match (&state.borrow::<SessionData<Session>>().csrf_token, state.borrow::<Request>().get_first("csrf-token")) {
            (&Some(ref session_token), Some(form_token)) if session_token != form_token => {
                info!(logger(), "CSRF validation failed, session token doesn't match the form token"; "session" => session_token, "form" => form_token);
                false
            },
            (&Some(_), Some(_)) => true,
            (session_token, form_token) => {
                info!(logger(), "CSRF validation failed, a token is missing"; "session" => ?session_token, "form" => ?form_token);
                false
            }
        }
    }

    pub fn generate_token(&self, state: &mut State) -> String {
        let mut rng = thread_rng();
        // 256 bits / log_2 (26 + 26 + 10) ≈ 42.99 symbols
        let token = (0..43).map(|_| rng.sample(Alphanumeric)).collect::<String>();
        state.borrow_mut::<SessionData<Session>>().csrf_token = Some(token.clone());
        token
    }
}

impl Middleware for CsrfMiddleware {
    fn call<Chain>(self, mut state: State, chain: Chain) -> Box<HandlerFuture>
    where
        Chain: FnOnce(State) -> Box<HandlerFuture> + 'static,
    {
        if self.is_post(&state) && (!self.check_headers(&state) || !self.check_token(&state)) {
            let context = Context::new(&mut state);
            let response = match state.borrow::<Templates>().render("csrf-validation-error.html", &context) {
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

        state.put(self.clone());

        chain(state)
    }
}
