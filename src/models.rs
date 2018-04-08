use chrono::{DateTime, Utc};
use schema::{audit_log, sessions, users};

use rand::{thread_rng, Rng};

#[derive(Identifiable, Queryable, Debug)]
pub struct Session {
    pub id: String,
    pub data: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, Debug)]
#[table_name = "sessions"]
pub struct NewSession<'a> {
    pub id: &'a str,
    pub data: &'a [u8],
}

#[derive(Identifiable, Queryable, Debug, StateData, Serialize, Clone)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub password: Vec<u8>,
    pub groups: Vec<String>,
    #[serde(skip)]
    pub created_at: DateTime<Utc>,
    #[serde(skip)]
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn verify_password(&self, password: &str) -> Result<(), ()> {
        if ::argon2rs::verifier::Encoded::from_u8(&self.password)
            .expect("failed to decode password hash")
            .verify(password.as_bytes())
        {
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn has_group(&self, group: &str) -> bool {
        self.groups.iter().any(|user_group| user_group == group)
    }
}

#[derive(Insertable, Debug)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub name: &'a str,
    pub password: Vec<u8>,
    pub groups: &'a [String],
}

impl<'a> NewUser<'a> {
    pub fn new(name: &'a str, password: &'a str, groups: &'a [String]) -> NewUser<'a> {
        NewUser {
            name,
            password: NewUser::hash_password(name, password),
            groups: groups,
        }
    }

    fn hash_password(name: &str, password: &str) -> Vec<u8> {
        let mut salt = vec![0u8; 32];
        thread_rng().fill(&mut salt[..]);
        ::argon2rs::verifier::Encoded::new(
            ::argon2rs::Argon2::default(::argon2rs::Variant::Argon2id),
            password.as_bytes(),
            &salt,
            &[],
            name.as_bytes(),
        ).to_u8()
    }
}

#[derive(Insertable, Debug)]
#[table_name = "audit_log"]
pub struct NewAuditLogEntry<'a> {
    pub user_id: i64,
    pub index: &'a str,
    pub query: &'a str,
}

impl<'a> NewAuditLogEntry<'a> {
    pub fn new(user_id: i64, index: &'a str, query: &'a str) -> NewAuditLogEntry<'a> {
        NewAuditLogEntry {
            user_id,
            index,
            query,
        }
    }
}
