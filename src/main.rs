#[macro_use] extern crate gotham_derive;
#[macro_use] extern crate json_str;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate diesel;
#[macro_use] extern crate slog;

extern crate argon2rs;
extern crate chrono;
extern crate elastic_reqwest;
extern crate gotham;
extern crate reqwest;
extern crate serde;
extern crate hyper;
extern crate futures;
extern crate phf;
extern crate rand;
extern crate tera;
extern crate url;

extern crate slog_term;
extern crate slog_async;
extern crate slog_scope;
extern crate slog_stdlog;

use gotham::router::builder::DrawRoutes;
use gotham::router::builder::DefineSingleRoute;
use slog::Drain;

macro_rules! try_h {
    ($state:expr, $e:expr) => ({
        let result = $e;
        match result {
            Ok(o) => o,
            Err(e) => return Box::new(::futures::IntoFuture::into_future(Err(($state, ::gotham::handler::IntoHandlerError::into_handler_error(e))))),
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
            .build(diesel::r2d2::ConnectionManager::new("postgresql:///panopticon"))
            .unwrap();

        let (chain, pipelines) = gotham::pipeline::single::single_pipeline(
                gotham::pipeline::new_pipeline()
                    .add(middleware::diesel::DieselMiddleware::new(pool.clone()))
                    .add(middleware::request::RequestParser::new())
                    .add(middleware::templates::Templates::new("templates/*").expect("failed to load templates"))
                    .add(gotham::middleware::session::NewSessionMiddleware::new(session_backend::PostgresBackend::new(pool.clone())).with_session_type::<Session>())
                    .add(middleware::csrf::CsrfMiddleware::new())
                    .add(middleware::user::UserMiddleware::new())
                    .build()
            );

        Ok(gotham::router::builder::build_router(chain, pipelines, |router| {
            router.get("/login").to(endpoints::login::form);
            router.post("/login").to(endpoints::login::login);
            router.post("/logout").to(endpoints::login::logout);
            router.get("/static/:file").with_path_extractor::<endpoints::static_files::PathParams>().to(endpoints::static_files::static_files);
        }))
    });
}
