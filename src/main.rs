#[macro_use]
extern crate diesel;
#[macro_use]
extern crate gotham_derive;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate slog;

extern crate argon2rs;
extern crate chrono;
extern crate elastic_reqwest;
extern crate futures;
extern crate gotham;
extern crate hyper;
extern crate phf;
extern crate rand;
extern crate reqwest;
extern crate serde;
extern crate slog_async;
extern crate slog_scope;
extern crate slog_stdlog;
extern crate slog_term;
extern crate tera;
extern crate tokio_core;
extern crate url;

use gotham::router::builder::DrawRoutes;
use gotham::router::builder::DefineSingleRoute;
use slog::Drain;
use futures::IntoFuture;

macro_rules! try_h {
    ($state:expr, $e:expr) => ({
        let result = $e;
        match result {
            Ok(o) => o,
            Err(e) => return Box::new(::futures::IntoFuture::into_future(Err(
                (
                    $state,
                    ::gotham::handler::IntoHandlerError::into_handler_error(e),
                )
            ))),
        }
    })
}

pub mod endpoints;
pub mod middleware;
pub mod models;
pub mod schema;
pub mod session_backend;
pub mod tera_helpers;

// WARNING: the session struct can't be changed without deleting all sessions.
#[derive(Debug, Default, Serialize, Deserialize)]
struct Session {
    user_id: Option<i64>,
    csrf_token: Option<String>,
    flashes: Vec<String>,
}

impl Session {
    pub fn flash<S: Into<String>>(&mut self, message: S) {
        self.flashes.push(message.into());
    }
}

#[derive(Debug, Serialize)]
struct Context {
    user: Option<models::User>,
    csrf_token: Option<String>,
    flashes: Vec<String>,

    #[serde(flatten)]
    extra: std::collections::HashMap<String, tera::Value>,
}

impl Context {
    pub fn new(state: &mut gotham::state::State) -> Context {
        let csrf = state.take::<middleware::csrf::CsrfMiddleware>();
        csrf.generate_token(state);
        state.put(csrf);

        let user = state.try_borrow::<models::User>().cloned();

        let session = state.borrow_mut::<gotham::middleware::session::SessionData<Session>>();

        Context {
            user,
            csrf_token: session.csrf_token.clone(),
            flashes: std::mem::replace(&mut session.flashes, vec![]),
            extra: std::collections::HashMap::new(),
        }
    }

    pub fn set<T: serde::Serialize, S: Into<String>>(&mut self, key: S, value: T) {
        self.extra.insert(
            key.into(),
            tera::to_value(value).expect("failed to convert to a Value"),
        );
    }
}

#[derive(Copy, Clone)]
struct Authenticated<T>(T);

impl<T: gotham::handler::Handler> gotham::handler::Handler for Authenticated<T> {
    fn handle(self, state: gotham::state::State) -> Box<gotham::handler::HandlerFuture> {
        if state.try_borrow::<models::User>().is_none() {
            let response =
                gotham::http::response::create_response(&state, hyper::StatusCode::SeeOther, None);
            Box::new(
                Ok((
                    state,
                    response.with_header(hyper::header::Location::new("/login")),
                )).into_future(),
            )
        } else {
            self.0.handle(state)
        }
    }
}

fn main() {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let logger = slog::Logger::root(drain, slog_o!("version" => env!("CARGO_PKG_VERSION")));

    let _scope_guard = slog_scope::set_global_logger(logger);
    let _log_guard = slog_stdlog::init().unwrap();

    gotham::start("[::1]:5200", || {
        let pool = diesel::r2d2::Pool::builder()
            .build(diesel::r2d2::ConnectionManager::new(
                "postgresql:///panopticon",
            ))
            .unwrap();

        let (chain, pipelines) = gotham::pipeline::single::single_pipeline(
            gotham::pipeline::new_pipeline()
                .add(middleware::error::ErrorMiddleware::new())
                .add(middleware::diesel::DieselMiddleware::new(pool.clone()))
                .add(middleware::request::RequestParser::new())
                .add(
                    middleware::templates::Templates::new("templates/*")
                        .expect("failed to load templates"),
                )
                .add(
                    gotham::middleware::session::NewSessionMiddleware::new(
                        session_backend::PostgresBackend::new(pool.clone()),
                    ).insecure()
                        .with_session_type::<Session>(),
                )
                .add(middleware::csrf::CsrfMiddleware::new())
                .add(middleware::user::UserMiddleware::new())
                .build(),
        );

        Ok(gotham::router::builder::build_router(
            chain,
            pipelines,
            |router| {
                router.get("/login").to(endpoints::login::form);
                router.post("/login").to(endpoints::login::login);
                router.post("/logout").to(endpoints::login::logout);
                router
                    .get("/static/:file")
                    .with_path_extractor::<endpoints::static_files::PathParams>()
                    .to(endpoints::static_files::static_files);
                router.get("/").to(Authenticated(endpoints::search::index));
                router
                    .post("/")
                    .to(Authenticated(endpoints::search::search));
            },
        ))
    });
}
