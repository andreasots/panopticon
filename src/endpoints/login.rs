use diesel::prelude::*;
use futures::{Future, IntoFuture};
use gotham::handler::{HandlerFuture, IntoHandlerError};
use gotham::middleware::session::SessionData;
use gotham::state::State;
use gotham_serde_json_body_parser::JSONBody;
use hyper::{Response, StatusCode};
use hyper::header::ContentType;

use middleware::diesel::DieselPool;
use models::User;
use ::Session;

#[derive(Deserialize)]
struct LoginRequest {
    user: String,
    password: String,
}

pub fn login_handler(state: State) -> Box<HandlerFuture> {
    use schema::users::dsl::*;

    let conn = match state.borrow::<DieselPool>().get() {
        Ok(conn) => conn,
        Err(err) => return Box::new(Err((state, err.into_handler_error())).into_future()),
    };

    Box::new(state.json::<LoginRequest>()
        .and_then(move |(mut state, req)| {
            let user = match users.filter(name.eq(&req.user)).first::<User>(&conn).optional() {
                Ok(Some(user)) => user,
                Ok(None) => return Ok((state, Response::new().with_status(StatusCode::Forbidden).with_header(ContentType::json()).with_body(json_str!({"errors": ["Incorrect username or password."]})))),
                Err(err) => return Err((state, err.into_handler_error())),
            };

            match user.verify_password(&req.password) {
                Ok(()) => (),
                Err(()) => return Ok((state, Response::new().with_status(StatusCode::Forbidden).with_header(ContentType::json()).with_body(json_str!({"errors": ["Incorrect username or password."]})))),
            }

            state.borrow_mut::<SessionData<Session>>().user_id = Some(user.id);

            Ok((state, Response::new().with_status(StatusCode::NoContent)))
        }))
}

pub fn logout_handler(mut state: State) -> (State, Response) {
    state.borrow_mut::<SessionData<Session>>().user_id = None;

    (state, Response::new().with_status(StatusCode::NoContent))
}

pub fn logged_in_handler(state: State) -> (State, Response) {
    let is_logged_in = state.borrow::<SessionData<Session>>().user_id.is_some();
    (state, if is_logged_in {
        Response::new()
            .with_status(StatusCode::NoContent)
    } else {
        Response::new()
            .with_status(StatusCode::Forbidden)
            .with_header(ContentType::json())
            .with_body(json_str!({"errors": ["Not logged in."]}))
    })
}
