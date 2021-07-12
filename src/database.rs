use postgres::Client;
use postgres_types::{ToSql, FromSql};
use tokio_postgres::{NoTls, Error};
use std::path::PathBuf;
use crate::content::Content;
use tlm::print::{self, print};

fn client_connection() -> Result<Client, Error> {
    let connection_string = r"postgresql://localhost:4531/tlmdb?user=postgres&password=786D3JXegfY8uR6shcPB7UF2oVeQf49ynH8vHgn".to_string();
    return Client::connect(&connection_string, NoTls);
}

pub fn db_connect(content: Content) -> Result<(), Error> {    
    //postgres
    let client_temp = client_connection();
    match client_temp {
        Err(err) => {
            print(print::Verbosity::ERROR, "db_connect", err.to_string());
        }
        _ => {
            let mut client = client_temp.unwrap();

            //client.batch_execute("DROP TABLE IF EXISTS content")?;
            //I want the auto generated ID of the entry
            let error = client.batch_execute("
                CREATE TABLE IF NOT EXISTS content (
                    uid             SERIAL PRIMARY KEY,
                    full_path       TEXT NOT NULL
                )
            ");

            match error {
                Err(err) => {
                    println!("{}", err.to_string());
                }
                _ => {

                }
            }

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
        }
    }
    

    

    

    Ok(())
}