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
        let connection_string = r"postgresql://localhost:4531/tlmdb?user=postgres&password=786D3JXegfY8uR6shcPB7UF2oVeQf49ynH8vHgn".to_string();
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

pub mod ensure {
    use crate::{database::execution::execute_query, utility::Utility};

    pub fn ensure_tables_exist(utility: Utility) {
        let utility = utility.clone_and_add_location("db_table_create");

        execute_query(
            r"
            CREATE TABLE IF NOT EXISTS content (
                content_uid     SERIAL PRIMARY KEY,
                full_path       TEXT NOT NULL,
                designation     INTEGER NOT NULL
            )",
            utility.clone(),
        );

        execute_query(
            r"
            CREATE TABLE IF NOT EXISTS show (
                show_uid        SERIAL PRIMARY KEY,
                title           TEXT NOT NULL
            )",
            utility.clone(),
        );

        execute_query(
            r"
            CREATE TABLE IF NOT EXISTS episode (
                content_uid             INTEGER REFERENCES content (content_uid) NOT NULL,
                show_uid                INTEGER REFERENCES show (show_uid) NOT NULL,
                episode_title           TEXT NOT NULL,
                season_number           SMALLINT NOT NULL,
                episode_number          SMALLINT NOT NULL,
                PRIMARY KEY(content_uid, show_uid, season_number, episode_number)
            )",
            utility.clone(),
        );

        execute_query(
            r"
            CREATE TABLE IF NOT EXISTS job_queue (
                job_uid             SERIAL PRIMARY KEY,
                source_path         TEXT NOT NULL,
                encode_path         TEXT NOT NULL,
                cache_directory     TEXT NOT NULL,
                encode_string       TEXT NOT NULL,
                status_underway     BOOLEAN NOT NULL,
                status_completed    BOOLEAN NOT NULL,
                worker_uid          INTEGER NOT NULL,
                worker_string_id    TEXT NOT NULL
            )",
            utility.clone(),
        );

        execute_query(
            r"
            CREATE TABLE IF NOT EXISTS job_task_queue (
                id                  INTEGER NOT NULL,
                job_uid             INTEGER REFERENCES job_queue (job_uid) NOT NULL,
                task_id             SMALLINT NOT NULL,
                PRIMARY KEY(job_uid, id)
            );",
            utility.clone(),
        );
    }
}

pub mod insert {
    use crate::{
        content::Content,
        database::{
            error_handling::handle_insert_error, execution::get_client,
            retrieve::get_uid_from_result,
        },
        utility::Utility,
    };

    pub fn insert_episode_if_episode(content: Content, utility: Utility) {
        let utility = utility.clone_and_add_location("insert_episode_if_episode");

        if content.content_is_episode() {
            let content_uid = content.content_uid.unwrap() as i32;
            let show_uid = content.show_uid.unwrap() as i32;
            let (season_number_temp, episode_number_temp) = content.show_season_episode.unwrap();
            let season_number = season_number_temp as i16;
            let episode_number = episode_number_temp[0] as i16; //asdf;
            let error = get_client(utility.clone()).execute(
                r"INSERT INTO episode (content_uid, show_uid, episode_title, episode_number, season_number) VALUES ($1, $2, $3, $4, $5)",
                &[&content_uid, &show_uid, &content.show_title.unwrap(), &episode_number, &season_number],
            );
            handle_insert_error(error, utility.clone());
        }
    }

    pub fn insert_content(content: Content, utility: Utility) -> usize {
        let utility = utility.clone_and_add_location("insert_content");

        let designation = content.designation as i32;
        let mut client = get_client(utility.clone());
        let content_uid = get_uid_from_result(client.query(
                r"INSERT INTO content (full_path, designation) VALUES ($1, $2) RETURNING content_uid;",
                &[&content.get_full_path(), &designation],
            ),
            utility.clone(),
        );
        return content_uid;
    }

    /* fn insert_task(task_id: usize, id: usize, job_uid: usize, utility: Utility) {
        let utility = utility.clone_and_add_location("insert_task");

        let mut client = get_client(utility.clone());
        let id = id as i32;
        let job_uid = job_uid as i32;
        let task_id = task_id as i16;
        let error = client.execute(
            r"INSERT INTO job_task_queue (
                    id,
                    job_uid,
                    task_id
                ) VALUES ($1, $2, $3)",
            &[&id, &job_uid, &task_id],
        );
        handle_insert_error(error, utility.clone());
        print(
            Verbosity::INFO,
            From::DB,
            utility.clone(),
            format!("[job_uid: {}][id: {}][task_id: {}]", job_uid, id, task_id),
            0,
        );
    }

    pub fn insert_job(job: Job, utility: Utility) {
        let utility = utility.clone_and_add_location("insert_job");

        let uid = insert_job_internal(job, utility.clone());

        fn insert_job_internal(job: Job, utility: Utility) -> usize {
            let utility = utility.clone_and_add_location("insert_job_internal");

            //get client and inserts job if the client connection is fine
            let mut client = get_client(utility.clone().clone());
            //quick retrieve ID
            let qrid = generate_qrid();
            let worker_uid = job.worker.clone().unwrap().0 as i32;
            let worker_string_identifier = job.worker.unwrap().1;
            handle_insert_error(
                client.execute(
                    r"
                    INSERT INTO job_queue (
                        source_path,
                        encode_path,
                        cache_directory,
                        encode_string,
                        status_underway,
                        status_completed,
                        worker_uid,
                        worker_string_id,
                        qrid
                    ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9);",
                    &[
                        &job.source_path.to_string_lossy().to_string().as_str(),
                        &job.encode_path.to_string_lossy().to_string().as_str(),
                        &job.cache_directory.clone().unwrap(),
                        &Job::convert_encode_string_to_actual_string(job.encode_string.clone()),
                        &job.status_underway,
                        &job.status_completed,
                        &worker_uid,
                        &worker_string_identifier,
                        &qrid,
                    ],
                ),
                utility.clone(),
            );
            let uid = read_back_job_uid(qrid, utility.clone());
            print(
                Verbosity::INFO,
                From::DB,
                utility.clone(),
                format!(
                    "[job_uid: {}][Source: {}][Encode: {}]",
                    uid,
                    job.source_path.to_string_lossy().to_string(),
                    job.encode_path.to_string_lossy().to_string()
                ),
                0,
            );
            for (pos, task) in job.tasks.iter().enumerate() {
                insert_task(task.clone() as usize, pos, uid, utility.clone());
            }
            return uid;
        }

        fn read_back_job_uid(qrid: i32, utility: Utility) -> usize {
            let utility = utility.clone_and_add_location("read_back_job_uid");

            return get_uid_from_result(
                handle_result_error(
                    get_client(utility.clone())
                        .query(r"SELECT job_uid from job_queue WHERE qrid = $1", &[&qrid]),
                    utility.clone(),
                ),
                utility,
            );
        }
    } */
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

pub mod miscellaneous {
    use crate::{database::execution::execute_query, utility::Utility};

    pub fn db_purge(utility: Utility) {
        let utility = utility.clone_and_add_location("db_purge");

        //the order for dropping tables matters if foreign keys exist (job_task_queue has a foreign key of job_queue)
        let tables: Vec<&str> = vec![
            "content",
            "job_task_queue",
            "job_queue",
            "episode",
            "season",
            "show",
        ];
        for table in tables {
            execute_query(
                &format!("DROP TABLE IF EXISTS {} CASCADE", table),
                utility.clone(),
            )
        }
    }
}

pub mod print {
    use crate::{
        content::Content,
        database::execution::get_by_query,
        print::{print, From, Verbosity},
        utility::Utility,
        show::Show,
    };

    pub fn print_jobs(utility: Utility) {
        let utility = utility.clone_and_add_location("print_jobs");

        for row in get_by_query(r"SELECT job_uid FROM job_queue", utility.clone()) {
            let uid: i32 = row.get(0);
            print(
                Verbosity::INFO,
                From::DB,
                utility.clone(),
                format!("[job_uid:{}]", uid),
                0,
            );
        }
    }

    pub fn print_shows(shows: Vec<Show>, utility: Utility) {
        let utility = utility.clone_and_add_location("print_shows");

        for show in shows {
            print(
                Verbosity::INFO,
                From::DB,
                utility.clone(),
                format!("[title:{}]", show.title),
                0,
            );
        }
    }

    pub fn print_contents(contents: Vec<Content>, utility: Utility) {
        let utility = utility.clone_and_add_location("print_contents");

        for content in contents {
            content.print(utility.clone());
        }
    }
}
