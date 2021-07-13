use postgres::Client;
use postgres_types::{ToSql, FromSql};
use tokio_postgres::{NoTls, Error};
use std::path::PathBuf;
use crate::content::Content;
use tlm::print::{self, print};

fn client_connection() -> Option<Client> {
    let connection_string = r"postgresql://localhost:4531/tlmdb?user=postgres&password=786D3JXegfY8uR6shcPB7UF2oVeQf49ynH8vHgn".to_string();
    let client = Client::connect(&connection_string, NoTls);
    match client {
        Err(err) => {
            print(print::Verbosity::ERROR, "client_connection", err.to_string());
            return None;
        }
        _ => {
            return Some(client.unwrap());
        }
    }
}

fn execute_query(query: &str) {
    let client = client_connection();
    if client.is_some() {
        let mut client = client.unwrap();
        client.batch_execute(query);
    }
}

//i want the auto generated ID of the entry
pub fn db_insert_content(content: Content) {
    execute_query(r"
        CREATE TABLE IF NOT EXISTS content (
            uid             SERIAL PRIMARY KEY,
            full_path       TEXT NOT NULL
        )
    ");
    let client = client_connection();
    if client.is_some() {
        let error = client.unwrap().execute(
            "INSERT INTO content (full_path) VALUES ($1)",
            &[&content.get_full_path()],
        );
    }
}

pub fn db_purge() {
    execute_query(r"DROP TABLE IF EXISTS content");
}

pub fn print_content() {
    let client = client_connection();
    if client.is_some() {
        let mut client = client.unwrap();
        for row in client.query(r"SELECT uid, full_path FROM content", &[]).unwrap() {
            let uid_temp: i32 = row.get(0);
            let uid = uid_temp as usize;
            let full_path_temp: String = row.get(1);
            let full_path = PathBuf::from(&full_path_temp);
            tlm::print::print(tlm::print::Verbosity::DEBUG, "DB", format!("{:3}:{}", uid, full_path_temp))
        }
    }
}
