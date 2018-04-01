use chrono::{DateTime, Utc};
use schema::{sessions, users};

use rand::Rng;

#[derive(Identifiable, Queryable, Debug)]
pub struct Session {
    pub id: String,
    pub data: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, Debug)]
#[table_name="sessions"]
pub struct NewSession<'a> {
    pub id: &'a str,
    pub data: &'a [u8],
}

#[derive(Identifiable, Queryable, Debug, StateData)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub password: Vec<u8>,
    pub groups: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn verify_password(&self, password: &str) -> Result<(), ()> {
        if ::argon2rs::verifier::Encoded::from_u8(&self.password).expect("failed to decode password hash").verify(password.as_bytes()) {
            Ok(())
        } else {
            Err(())
        }
    }
}

#[derive(Insertable, Debug)]
#[table_name="users"]
pub struct NewUser<'a> {
    pub name: &'a str,
    pub password: Vec<u8>,
    pub groups: &'a [String],
}

impl<'a> NewUser<'a> {
    pub fn new(name: &'a str, password: &str, groups: &'a [String]) -> NewUser<'a> {
        NewUser {
            name,
            password: NewUser::hash_password(name, password),
            groups: groups,
        }
    }

    fn hash_password(name: &str, password: &str) -> Vec<u8> {
        let mut salt = vec![0u8; 32];
        ::rand::OsRng::new().expect("failed to open the OS randomness source").fill(&mut salt[..]);
        ::argon2rs::verifier::Encoded::new(::argon2rs::Argon2::default(::argon2rs::Variant::Argon2id), password.as_bytes(), &salt, &[], name.as_bytes()).to_u8()
    }
}
