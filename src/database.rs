use crate::content::{Content, Job, Task};
use postgres::Client;
use postgres_types::{FromSql, ToSql};
use std::path::PathBuf;
use tlm::print::{self, print};
use tokio_postgres::{Error, NoTls, Row};

fn client_connection() -> Option<Client> {
    let connection_string = r"postgresql://localhost:4531/tlmdb?user=postgres&password=786D3JXegfY8uR6shcPB7UF2oVeQf49ynH8vHgn".to_string();
    let client = Client::connect(&connection_string, NoTls);
    match client {
        Err(err) => {
            print(
                print::Verbosity::ERROR,
                "client_connection",
                err.to_string(),
            );
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
    execute_query(
        r"
        CREATE TABLE IF NOT EXISTS content (
            uid             SERIAL PRIMARY KEY,
            full_path       TEXT NOT NULL
        )",
    );
    let client = client_connection();
    if client.is_some() {
        let error = client.unwrap().execute(
            r"INSERT INTO content (full_path) VALUES ($1)",
            &[&content.get_full_path()],
        );
    }
}

pub fn db_insert_task(task_id: usize, id: usize, job_uid: usize) {
    //pull out in order by id

    execute_query(
        r"
        CREATE TABLE IF NOT EXISTS job_task_queue (
            id                  INTEGER PRIMARY KEY,
            job_uid             INTEGER FOREIGN KEY,
            task_id             SMALLINT
        )",
    );

    let client = client_connection();
    if client.is_some() {
        let id = id as i32;
        let job_uid = job_uid as i32;
        let task_id = task_id as i32;

        let error = client.unwrap().execute(
            r"INSERT INTO job_task_queue (
                    id,
                    job_uid,
                    task_uid,
                ) VALUES ($1, $2, $3)",
            &[&id, &job_uid, &task_id],
        );
    }
}

pub fn db_insert_job(job: Job) {
    //ensures job table exists
    execute_query(
        r"
        CREATE TABLE IF NOT EXISTS job_queue (
            uid                 SERIAL PRIMARY KEY,
            source_path         TEXT NOT NULL,
            encode_path         TEXT NOT NULL,
            encode_string       TEXT NOT NULL,
            cache_directory     TEXT NOT NULL,
            status_underway     BOOLEAN NOT NULL,
            status_completed    BOOLEAN NOT NULL,
            worker_uid          INTEGER NOT NULL,
            worker_string_id    TEXT NOT NULL,
            full_path           TEXT NOT NULL,
            qrid                INTEGER NOT NULL
        )",
    );

    //get client and inserts job if the client connection is fine
    let client = client_connection();
    if client.is_some() {
        let qrid = 390192782;
        let worker_uid = job.clone().worker.clone().unwrap().0 as i32;
        let worker_string_identifier = job.worker.unwrap().1;
        let error = client.unwrap().execute(
            r"
                INSERT INTO job_queue (
                    source_path,
                    encode_path,
                    encode_string,
                    cache_directory,
                    status_underway,
                    status_completed,
                    worker_uid,
                    worker_string_identifier,
                    qrid
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            &[
                &job.source_path.to_string_lossy().to_string(),
                &job.encode_path.to_string_lossy().to_string(),
                &job.encode_string,
                &job.cache_directory,
                &job.status_underway,
                &job.status_completed,
                &worker_uid,
                &worker_string_identifier,
                &qrid,
            ],
        );
        let client = client_connection();
        if client.is_some() {
            //i only want the latest one that fits this query to contend with the potential statistical crossover
            let result = client
                .unwrap()
                .query(r"SELECT uid from job_queue WHERE qrid = $1", &[&qrid]);
            if result.is_ok() {
                //let t = *result.unwrap().get(0).unwrap();
                //let f = result.unwrap()[0].get(0);

                let uid_temp: i32 = result.unwrap()[0].get(0);
                let uid: usize = uid_temp as usize;

                for (pos, task) in job.tasks.iter().enumerate() {
                    db_insert_task(task.clone() as usize, pos, uid);
                }
            }
        }
    }
}

pub fn db_get_by_query(query: &str) -> Option<Result<Vec<Row>, Error>> {
    let client = client_connection();
    if client.is_some() {
        return Some(client.unwrap().query(query, &[]));
    }
    return None;
}

pub fn db_purge() {
    execute_query(r"DROP TABLE IF EXISTS content");
    execute_query(r"DROP TABLE IF EXISTS job_queue");
    execute_query(r"DROP TABLE IF EXISTS job_task_queue");
}

pub fn print_jobs() {
    let result = db_get_by_query(r"SELECT id, job_uid, task_id FROM job_queue");
    if result.is_some() {
        let result = result.unwrap();
        if result.is_ok() {
            let result = result.unwrap();
            for row in result {
                let id: i32 = row.get(0);
                let job_uid: i32 = row.get(1);
                let task_id: i32 = row.get(2);
                tlm::print::print(
                    tlm::print::Verbosity::DEBUG,
                    "db",
                    format!("[job_uid: {}][id: {}][task_id: {}]", job_uid, id, task_id),
                )
            }
        }
    }
}

pub fn print_content() {
    let result = db_get_by_query(r"SELECT uid, full_path FROM content");
    if result.is_some() {
        let result = result.unwrap();
        if result.is_ok() {
            let result = result.unwrap();
            for row in result {
                let uid_temp: i32 = row.get(0);
                let uid = uid_temp as usize;
                let full_path_temp: String = row.get(1);
                let full_path = PathBuf::from(&full_path_temp);
                tlm::print::print(
                    tlm::print::Verbosity::DEBUG,
                    "DB",
                    format!("{:3}:{}", uid, full_path_temp),
                )
            }
        }
    }
}
