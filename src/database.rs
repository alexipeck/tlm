pub mod error_handling {
    use crate::{
        print::{print, From, Verbosity},
        utility::Utility,
    };
    use tokio_postgres::{Error, Row};

    pub fn handle_result_error(result: Result<Vec<Row>, Error>, utility: Utility) -> Vec<Row> {
        let utility = utility.clone_and_add_location("handle_result_error");

        if result.is_ok() {
            let result = result.unwrap();
            if result.len() > 0 {
                return result;
            }
        } else {
            handle_retrieve_error(result, utility.clone());
        }
        return Vec::new();
    }

    //prints error of it's actually an error, otherwise, does nothing
    pub fn handle_insert_error(error: Result<u64, Error>, utility: Utility) {
        let utility = utility.clone_and_add_location("handle_insert_error");

        if error.is_err() {
            print(
                Verbosity::ERROR,
                From::DB,
                utility,
                format!("{}", error.unwrap_err()),
                0,
            );
        }
    }

    //prints error of it's actually an error, otherwise, returns unwrapped Vec<Row>
    pub fn handle_retrieve_error(error: Result<Vec<Row>, Error>, utility: Utility) {
        let utility = utility.clone_and_add_location("handle_retrieve_error");

        if error.is_err() {
            print(
                Verbosity::ERROR,
                From::DB,
                utility,
                format!(
                    "something is wrong with the returned result, or lack their of: {}",
                    error.unwrap_err()
                ),
                0,
            );
            panic!();
        }
    }

    //given an error handled Vec<Row>, will return boolean or handle the error
    pub fn db_boolean_handle(input: Vec<Row>, utility: Utility) -> bool {
        let utility = utility.clone_and_add_location("db_boolean_handle");

        if input.len() > 0 {
            //requires explicit type
            if input[0].get(0) {
                return true;
            } else {
                return false;
            }
        } else {
            print(
                Verbosity::CRITICAL,
                From::DB,
                utility,
                format!("should have returned a boolean from the db, regardless"),
                0,
            );
            panic!();
        }
    }
}

pub mod execution {
    use crate::{
        database::error_handling::handle_result_error,
        print::{print, From, Verbosity},
        utility::Utility,
    };
    use postgres::Client;
    use tokio_postgres::{NoTls, Row};

    pub fn get_by_query(query: &str, utility: Utility) -> Vec<Row> {
        let utility = utility.clone_and_add_location("get_by_query");

        let result = get_client(utility.clone()).query(query, &[]);
        return handle_result_error(result, utility.clone());
    }

    //use enums for database insertion, with helper functions that allow me to directly pass in each variable

    //creates and returns a postgreSQL database client connection
    pub fn get_client(utility: Utility) -> Client {
        let utility = utility.clone_and_add_location("get_client");

        //credentials aren't secret yet, and are only for a testing/development database.
        let connection_string = r"postgresql://localhost:5432/tlmdb?user=postgres&password=786D3JXegfY8uR6shcPB7UF2oVeQf49ynH8vHgn".to_string();
        //creates actual database client connection
        //returns unhandled result with client
        let client = Client::connect(&connection_string, NoTls);
        //if there is an error, it's printed and panics, otherwise unwrapped
        match client {
            Err(err) => {
                print(
                    Verbosity::ERROR,
                    From::DB,
                    utility,
                    format!(
                        "client couldn't establish a connection: {}",
                        err.to_string()
                    ),
                    0,
                );
                panic!();
            }
            _ => {
                return client.unwrap();
            }
        }
    }

    //used for executing queries that return nothing, errors are handled internally
    pub fn execute_query(query: &str, utility: Utility) {
        let utility = utility.clone_and_add_location("execute_query");

        let mut client = get_client(utility.clone());
        //stores error returned by
        let error = client.batch_execute(query);
        if error.is_err() {
            print(
                Verbosity::ERROR,
                From::DB,
                utility.clone(),
                format!("{}: {}", String::from(query), error.unwrap_err()),
                0,
            );
        }
    }
}

pub mod retrieve {
    use crate::{database::error_handling::handle_result_error, utility::Utility};
    use tokio_postgres::{Error, Row};

    pub fn get_uid_from_result(result: Result<Vec<Row>, Error>, utility: Utility) -> usize {
        let utility = utility.clone_and_add_location("get_uid_from_result");

        let result: i32 = handle_result_error(result, utility.clone())[0].get(0);
        return result as usize;
    }
}