use gotham::handler::HandlerFuture;
use gotham::middleware::Middleware;
use gotham::state::State;
use hyper::Body;
use hyper::header::ContentType;
use hyper::Headers;
use url::form_urlencoded;
use futures::{Future, Stream};
use gotham::handler::IntoHandlerError;
use std::collections::HashMap;
use hyper::Error as HyperError;

#[derive(StateData)]
pub struct Request {
    data: HashMap<String, Vec<String>>,
}

impl Request {
    pub fn get_first(&self, key: &str) -> Option<&str> {
        self.data
            .get(key)
            .and_then(|values| values.get(0))
            .map(String::as_str)
    }
}

#[derive(Clone, NewMiddleware)]
pub struct RequestParser {}

impl RequestParser {
    pub fn new() -> RequestParser {
        RequestParser {}
    }
}

impl Middleware for RequestParser {
    fn call<Chain>(self, mut state: State, chain: Chain) -> Box<HandlerFuture>
    where
        Chain: FnOnce(State) -> Box<HandlerFuture> + 'static,
    {
        if state.borrow::<Headers>().get::<ContentType>() == Some(&ContentType::form_url_encoded())
        {
            Box::new(
                state
                    .take::<Body>()
                    .fold(vec![], |mut req, chunk| -> Result<Vec<u8>, HyperError> {
                        req.extend(chunk);
                        Ok(req)
                    })
                    .then(move |res| match res {
                        Ok(body) => {
                            let mut data = HashMap::new();

                            for (key, value) in form_urlencoded::parse(&body) {
                                data.entry(key.to_string())
                                    .or_insert_with(Vec::new)
                                    .push(value.to_string());
                            }

                            state.put(Request { data });

                            Ok(state)
                        }
                        Err(err) => Err((state, err.into_handler_error())),
                    })
                    .and_then(chain),
            )
        } else {
            chain(state)
        }
    }
}
