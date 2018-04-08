#[macro_use]
extern crate diesel;
#[macro_use]
extern crate gotham_derive;
#[macro_use]
extern crate serde_derive;

extern crate argon2rs;
extern crate chrono;
extern crate gotham;
extern crate rand;

// FIXME: reorganise? libpanopticon?
#[path = "../models.rs"]
pub mod models;
#[path = "../schema.rs"]
pub mod schema;

use std::io::Write;
use diesel::prelude::*;

fn prompt(prompt: &str) -> std::io::Result<String> {
    print!("{}: ", prompt);
    std::io::stdout().flush()?;
    let mut line = String::new();
    std::io::stdin().read_line(&mut line)?;
    Ok(line.trim().to_string())
}

fn main() {
    let name = prompt("Username").expect("failed to read the username");
    let password = prompt("Password").expect("failed to read the password");
    let mut groups = vec![];
    let mut group;
    while {
        group = prompt("Group (empty string to stop)").expect("failed to read the group");
        group.len() > 0
    } {
        groups.push(group);
    }

    let conn = diesel::PgConnection::establish("postgresql:///panopticon")
        .expect("failed to connect to the database");

    diesel::insert_into(schema::users::table)
        .values(&models::NewUser::new(&name, &password, &groups))
        .execute(&conn)
        .expect("failed to add the user");

    println!("Added {:?}", name);
}
