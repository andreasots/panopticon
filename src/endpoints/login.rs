use diesel::prelude::*;
use futures::IntoFuture;
use gotham::handler::HandlerFuture;
use gotham::middleware::session::SessionData;
use gotham::http::response::create_response;
use gotham::state::State;
use hyper::{Response, StatusCode};
use hyper::header::Location;
use hyper::mime::TEXT_HTML;

use middleware::diesel::DieselPool;
use middleware::request::Request;
use middleware::templates::Templates;
use models::User;
use Session;
use Context;

pub fn form(mut state: State) -> Box<HandlerFuture> {
    let context = Context::new(&mut state);
    let response = try_h!(
        state,
        state.borrow::<Templates>().render("login.html", &context)
    );
    let response = create_response(
        &state,
        StatusCode::Ok,
        Some((response.into_bytes(), TEXT_HTML)),
    );
    Box::new(Ok((state, response)).into_future())
}

pub fn login(mut state: State) -> Box<HandlerFuture> {
    use schema::users::dsl::*;

    let conn = try_h!(state, state.borrow::<DieselPool>().get());

    let (user, pass) = {
        let (user, pass) = {
            let req = state.borrow::<Request>();
            (
                req.get_first("username").map(String::from),
                req.get_first("password").map(String::from),
            )
        };
        match (user, pass) {
            (Some(user), Some(pass)) => (user, pass),
            (user, pass) => {
                state
                    .borrow_mut::<SessionData<Session>>()
                    .flash("Incorrect username or password.");
                let mut context = Context::new(&mut state);
                context.set("username", user);
                context.set("password", pass);
                let response = try_h!(
                    state,
                    state.borrow::<Templates>().render("login.html", &context)
                );
                let response = create_response(
                    &state,
                    StatusCode::BadRequest,
                    Some((response.into_bytes(), TEXT_HTML)),
                );
                return Box::new(Ok((state, response)).into_future());
            }
        }
    };

    let user = match try_h!(
        state,
        users.filter(name.eq(&user)).first::<User>(&conn).optional()
    ) {
        Some(user) => user,
        None => {
            state
                .borrow_mut::<SessionData<Session>>()
                .flash("Incorrect username or password.");
            let mut context = Context::new(&mut state);
            context.set("username", user);
            context.set("password", pass);
            let response = try_h!(
                state,
                state.borrow::<Templates>().render("login.html", &context)
            );
            let response = create_response(
                &state,
                StatusCode::BadRequest,
                Some((response.into_bytes(), TEXT_HTML)),
            );
            return Box::new(Ok((state, response)).into_future());
        }
    };

    match user.verify_password(&pass) {
        Ok(()) => (),
        Err(()) => {
            state
                .borrow_mut::<SessionData<Session>>()
                .flash("Incorrect username or password.");
            let mut context = Context::new(&mut state);
            context.set("username", user.name);
            context.set("password", pass);
            let response = try_h!(
                state,
                state.borrow::<Templates>().render("login.html", &context)
            );
            let response = create_response(
                &state,
                StatusCode::BadRequest,
                Some((response.into_bytes(), TEXT_HTML)),
            );
            return Box::new(Ok((state, response)).into_future());
        }
    }

    state.borrow_mut::<SessionData<Session>>().user_id = Some(user.id);
    state
        .borrow_mut::<SessionData<Session>>()
        .flash("Logged in successfully.");
    let response = create_response(&state, StatusCode::SeeOther, None);
    Box::new(Ok((state, response.with_header(Location::new("/")))).into_future())
}

pub fn logout(mut state: State) -> (State, Response) {
    {
        let session = state.borrow_mut::<SessionData<Session>>();
        session.user_id = None;
        session.flash("Logged out successfully");
    }

    let response = create_response(&state, StatusCode::SeeOther, None);
    (state, response.with_header(Location::new("/")))
}
