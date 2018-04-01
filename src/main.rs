#[macro_use] extern crate gotham_derive;
#[macro_use] extern crate json_str;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate diesel;

extern crate argon2rs;
extern crate chrono;
extern crate elastic_reqwest;
extern crate gotham;
extern crate gotham_serde_json_body_parser;
extern crate reqwest;
extern crate serde;
extern crate hyper;
extern crate futures;
extern crate rand;

use gotham::router::builder::DrawRoutes;
use gotham::router::builder::DefineSingleRoute;

pub mod endpoints;
pub mod middleware;
pub mod models;
pub mod schema;
pub mod session_backend;

// WARNING: the session struct can't be changed without deleting all sessions.
#[derive(Debug, Default, Serialize, Deserialize)]
struct Session {
    user_id: Option<i64>,
}

fn main() {
    gotham::start("[::1]:4100", || {
        let pool = diesel::r2d2::Pool::builder()
            .build(diesel::r2d2::ConnectionManager::new("postgresql:///panopticon"))
            .unwrap();

        let (chain, pipelines) = gotham::pipeline::single::single_pipeline(
                gotham::pipeline::new_pipeline()
                    .add(middleware::diesel::DieselMiddleware::new(pool.clone()))
                    .add(gotham::middleware::session::NewSessionMiddleware::new(session_backend::PostgresBackend::new(pool.clone())).with_session_type::<Session>())
                    .add(middleware::user::UserMiddleware::new())
                    .build()
            );

        Ok(gotham::router::builder::build_router(chain, pipelines, |router| {
            router.post("/api/v1/login").to(endpoints::login::login_handler);
            router.post("/api/v1/logout").to(endpoints::login::logout_handler);
            router.get("/api/v1/logged_in").to(endpoints::login::logged_in_handler);
        }))
    });
}
