use postgres::Client;
use postgres_types::{ToSql, FromSql};
use tokio_postgres::{NoTls, Error};
use std::path::PathBuf;
use crate::content::Content;
//use tlm::{print, Verbosity};

pub fn db_connect(content: Content) -> Result<(), Error> {//async
    /* async fn asynchonous_db_connect() -> Result<(), Error> {
        let (client, connection) = tokio_postgres::connect("host=localhost user=postgres", NoTls).await?;
        // The connection object performs the actual communication with the database,
        // so spawn it off to run on its own.
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        // Now we can execute a simple statement that just returns its parameter.
        let rows = client
            .query("SELECT $1::TEXT", &[&"hello world"])
            .await?;

        // And then check that we got back the same string we sent over.
        let value: &str = rows[0].get(0);
        assert_eq!(value, "hello world");

        Ok(())
    } */
    
    //postgres
    let connection_string = r"postgresql://localhost:4531/tlmdb?user=postgres&password=786D3JXegfY8uR6shcPB7UF2oVeQf49ynH8vHgn".to_string();
    let mut client = Client::connect(&connection_string, NoTls)?;

    client.batch_execute("DROP TABLE IF EXISTS content")?;
    //ensures table exists
    //I want the auto generated ID of the entry
    client.batch_execute("
        CREATE TABLE IF NOT EXISTS content (
            uid             SERIAL PRIMARY KEY,
            full_path       TEXT NOT NULL
        )
    ")?;
    
    //insert data
    client.execute(
        "INSERT INTO content (full_path) VALUES ($1)",
        &[&content.get_full_path()],
    )?;

    //read back
    for row in client.query("SELECT uid, full_path FROM content", &[])? {
        let uid_temp: i32 = row.get(0);
        let uid: usize = uid_temp as usize;
        let full_path_temp: String = row.get(1);//might need to be raw or something
        let full_path = PathBuf::from(full_path_temp);
        tlm::print::print(tlm::print::Verbosity::DEBUG, "DB", format!("{:3}:{}", uid, full_path.as_os_str().to_str().unwrap().to_string()))
    }

    Ok(())
}