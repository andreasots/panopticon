use diesel::prelude::*;
use futures::{Future, IntoFuture};
use gotham::handler::{HandlerFuture, IntoHandlerError};
use gotham::middleware::session::SessionData;
use gotham::http::response::create_response;
use gotham::state::State;
use hyper::{Response, StatusCode};
use hyper::header::ContentType;
use tera::Context;
use hyper::mime::TEXT_HTML;

use middleware::diesel::DieselPool;
use middleware::request::Request;
use middleware::templates::Templates;
use models::User;
use ::Session;

pub fn form(state: State) -> Box<HandlerFuture> {
    let response = try_h!(state, state.borrow::<Templates>().render("login.html", &Context::new()));
    let response = create_response(&state, StatusCode::Ok, Some((response.into_bytes(), TEXT_HTML)));
    Box::new(Ok((state, response)).into_future())
}

pub fn login(state: State) -> Box<HandlerFuture> {/*
    use schema::users::dsl::*;

    let conn = try_h!(state, state.borrow::<DieselPool>().get());

    let req = state.borrow::<Request>();

    let user = match try_h!(state, users.filter(name.eq(&req.get_first("username")).first::<User>(&conn).optional())) {
        Some(user) => user,
        None => return Box::new(Ok(state, Response::new().with_status(StatusCode::Forbidden).with_header(ContentType::json()).with_body(json_str!({"errors": ["Incorrect username or password."]}))).into_future()),
    };

    match user.verify_password(&req.get_first("password")) {
        Ok(()) => (),
        Err(()) => return Box::new(Ok((state, Response::new().with_status(StatusCode::Forbidden).with_header(ContentType::json()).with_body(json_str!({"errors": ["Incorrect username or password."]})))).into_future()),
    }

    state.borrow_mut::<SessionData<Session>>().user_id = Some(user.id);
*/
    let response = create_response(&state, StatusCode::NoContent, None);
    Box::new(Ok((state, response)).into_future())
}

pub fn logout(mut state: State) -> (State, Response) {
    state.borrow_mut::<SessionData<Session>>().user_id = None;

    let response = create_response(&state, StatusCode::NoContent, None);
    (state, response)
}
