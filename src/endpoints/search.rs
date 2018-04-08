use futures::IntoFuture;
use gotham::handler::{HandlerFuture, IntoHandlerError};
use gotham::http::response::create_response;
use gotham::state::State;
use hyper::StatusCode;
use hyper::mime::TEXT_HTML;
use gotham::middleware::session::SessionData;
use tokio_core::reactor::Handle;
use elastic_reqwest;
use elastic_reqwest::{AsyncElasticClient, AsyncFromResponse, parse};
use elastic_reqwest::req::SearchRequest;
use elastic_reqwest::res::SearchResponse;
use serde_json::Value;
use serde_json::map::Map;
use futures::Future;
use diesel;
use diesel::prelude::*;

use middleware::templates::Templates;
use middleware::request::Request;
use middleware::diesel::DieselPool;
use models::{NewAuditLogEntry, User};
use schema::audit_log;
use ::Context;
use ::Session;

const INDICES: [(&str, &str); 1] = [
    ("twitch", "Twitch messages"),
];

fn get_indices(user: &User) -> Vec<(&'static str, &'static str)> {
    let mut indices = INDICES.iter()
        .filter(|&&(name, _)| user.has_group(&format!("read-index.{}", name)))
        .cloned()
        .collect::<Vec<_>>();
    indices.sort();
    indices
}

pub fn index(mut state: State) -> Box<HandlerFuture> {
    let mut context = Context::new(&mut state);

    context.set("indices", get_indices(state.borrow::<User>()));

    let response = try_h!(state, state.borrow::<Templates>().render("index.html", &context));
    let response = create_response(&state, StatusCode::Ok, Some((response.into_bytes(), TEXT_HTML)));
    Box::new(Ok((state, response)).into_future())
}

fn clean_input(s: Option<&str>) -> Option<String> {
    s
        .map(str::trim)
        .and_then(|s| if s.len() > 0 { Some(s) } else { None })
        .map(String::from)
}

fn flatten(val: Value) -> Value {
    match val {
        Value::Object(obj) => Value::Object(flatten_object(obj, None)),
        value => value,
    }
}

fn flatten_object(obj: Map<String, Value>, prefix: Option<&str>) -> Map<String, Value> {
    let mut ret = Map::new();

    for (key, value) in obj {
        match value {
            Value::Object(obj) => ret.extend(flatten_object(obj, Some(&key))),
            value => {
                ret.insert(prefix.map(|prefix| format!("{}.{}", prefix, key)).unwrap_or(key), value);
            },
        }
    }

    ret
}

pub fn search(mut state: State) -> Box<HandlerFuture> {
    let (index, query) = {
        let req = state.borrow::<Request>();
        (clean_input(req.get_first("index")), clean_input(req.get_first("query")))
    };

    let (index, query) = match (index, query) {
        (Some(index), Some(query)) => (index, query),
        (index, query) => {
            state.borrow_mut::<SessionData<Session>>().flash("Required field missing.");

            let mut context = Context::new(&mut state);
            context.set("indices", get_indices(state.borrow::<User>()));
            context.set("index", index);
            context.set("query", query);

            let response = try_h!(state, state.borrow::<Templates>().render("index.html", &context));
            let response = create_response(&state, StatusCode::BadRequest, Some((response.into_bytes(), TEXT_HTML)));
            return Box::new(Ok((state, response)).into_future());
        }
    };

    let user_id = {
        let (user_id, allowed) = {
            let user = state.borrow::<User>();
            (user.id, user.has_group(&format!("read-index.{}", index)))
        };
        if ! allowed {
            state.borrow_mut::<SessionData<Session>>().flash("Access denied");
            let mut context = Context::new(&mut state);
            context.set("indices", get_indices(state.borrow::<User>()));
            context.set("index", &index);
            context.set("query", &query);
            let response = try_h!(state, state.borrow::<Templates>().render("index.html", &context));
            let response = create_response(&state, StatusCode::BadRequest, Some((response.into_bytes(), TEXT_HTML)));
            return Box::new(Ok((state, response)).into_future());
        }
        user_id
    };

    let mut context = Context::new(&mut state);
    context.set("indices", get_indices(state.borrow::<User>()));
    context.set("index", &index);
    context.set("query", &query);

    {
        let conn = try_h!(state, state.borrow::<DieselPool>().get());
        try_h!(state, diesel::insert_into(audit_log::table)
            .values(&NewAuditLogEntry::new(user_id, &index, &query))
            .execute(&conn));
    }

    let handle = state.borrow::<Handle>().clone();
    let (client, params) = try_h!(state, elastic_reqwest::async::default(&handle));

    let search = SearchRequest::for_index(
        index,
        json!({
            "size": 1000,
            "query": {
                "query_string" : {
                    "query": query,
                }
            }
        })
    );

    Box::new(client
        .elastic_req(&params, search)
        .and_then(|http_res| parse::<SearchResponse<Value>>().from_response(http_res))
        .then(move |res| {
            let res = res
                .map_err(IntoHandlerError::into_handler_error)
                .and_then(|response| {
                    context.set("hits", response.into_documents().map(flatten).collect::<Vec<_>>());
                    state.borrow::<Templates>().render("index.html", &context)
                        .map_err(IntoHandlerError::into_handler_error)
                })
                .map(|response| create_response(&state, StatusCode::BadRequest, Some((response.into_bytes(), TEXT_HTML))));
            match res {
                Ok(res) => Ok((state, res)),
                Err(e) => Err((state, e)),
            }
        }))
}
