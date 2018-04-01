use diesel;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::pg::upsert::excluded;
use futures::{Future, IntoFuture};
use gotham::middleware::session::{Backend, NewBackend, SessionError, SessionIdentifier};
use std::io::Error;
use std::panic::AssertUnwindSafe;

use models::{NewSession, Session};

pub struct PostgresBackend {
    // It's not actually RefUnwindSafe.
    pool: AssertUnwindSafe<Pool<ConnectionManager<PgConnection>>>,
}

impl PostgresBackend {
    pub fn new(pool: Pool<ConnectionManager<PgConnection>>) -> PostgresBackend {
        PostgresBackend {
            pool: AssertUnwindSafe(pool),
        }
    }

    fn get_conn(&self) -> Result<PooledConnection<ConnectionManager<PgConnection>>, SessionError> {
        self.pool.0.get()
            .map_err(|err| SessionError::Backend(format!("failed to check out a connection from the pool: {}", err)))
    }
}

impl Backend for PostgresBackend {
    fn persist_session(&self, identifier: SessionIdentifier, content: &[u8]) -> Result<(), SessionError> {
        use schema::sessions::dsl::*;

        let conn = self.get_conn()?;
        let new_session = NewSession {
            id: &identifier.value,
            data: content,
        };
        diesel::insert_into(sessions)
            .values(&new_session)
            .on_conflict(id)
            .do_update()
            .set(data.eq(excluded(data)))
            .execute(&conn)
            .map_err(|err| SessionError::Backend(format!("failed to update session: {}", err)))?;
        Ok(())
    }

    fn read_session(&self, identifier: SessionIdentifier) -> Box<Future<Item=Option<Vec<u8>>, Error=SessionError>> {
        use schema::sessions::dsl::*;

        Box::new(self.get_conn()
            .into_future()
            .and_then(move |conn|
                sessions.filter(id.eq(&identifier.value))
                    .first::<Session>(&conn)
                    .optional()
                    .map_err(|err| SessionError::Backend(format!("failed to read session: {}", err)))
            )
            .map(|session| session.map(|session| session.data))
        ) as Box<Future<Item=Option<Vec<u8>>, Error=SessionError>>
    }

    fn drop_session(&self, identifier: SessionIdentifier) -> Result<(), SessionError> {
        use schema::sessions::dsl::*;

        let conn = self.get_conn()?;
        diesel::delete(sessions.filter(id.eq(&identifier.value)))
            .execute(&conn)
            .map_err(|err| SessionError::Backend(format!("failed to delete session: {}", err)))?;
        Ok(())
    }
}

impl Clone for PostgresBackend {
    fn clone(&self) -> Self {
        PostgresBackend {
            pool: AssertUnwindSafe(self.pool.0.clone()),
        }
    }
}

impl NewBackend for PostgresBackend {
    type Instance = Self;
    fn new_backend(&self) -> Result<Self::Instance, Error> {
        Ok(PostgresBackend::new(self.pool.0.clone()))
    }
}
