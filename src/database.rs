use crate::print::{print, Verbosity, From};
use crate::{
    content::{Content, Job, Task},
    shows::{self, Show},
};
use core::panic;
use postgres::Client;
use postgres_types::{FromSql, ToSql};
use rand::Rng;
use std::path::PathBuf;
use tlm::designation::Designation;
use tokio_postgres::{Error, NoTls, Row};

//primary helper functions
fn generate_qrid() -> i32 {
    let mut rng = rand::thread_rng();
    let qrid_temp: u32 = rng.gen_range(0..2147483646);
    return qrid_temp as i32;
}

fn client_connection() -> Option<Client> {
    let connection_string = r"postgresql://localhost:4531/tlmdb?user=postgres&password=786D3JXegfY8uR6shcPB7UF2oVeQf49ynH8vHgn".to_string();
    let client = Client::connect(&connection_string, NoTls);
    match client {
        Err(err) => {
            print(Verbosity::ERROR, From::DB, "client_connection", err.to_string());
            return None;
        }
        _ => {
            return Some(client.unwrap());
        }
    }
}

fn output_insert_error(error: Result<u64, Error>, function_called_from: &str) {
    if error.is_err() {
        print(
            Verbosity::ERROR,
            From::DB,
            function_called_from,
            format!("{}", error.unwrap_err()),
        );
    }
}

fn output_retrieve_error(error: Result<Vec<Row>, Error>, function_called_from: &str) {
    if error.is_err() {
        print(
            Verbosity::ERROR,
            From::DB,
            function_called_from,
            format!("{}", error.unwrap_err()),
        );
    }
}

//template
/* pub struct Season {
    pub number: usize,
    pub episodes: Vec<Content>,
}

pub struct Show {
    pub uid: usize,
    pub title: String,
    pub seasons: Vec<Season>,
}

pub struct Shows {
    pub shows: Vec<Show>,
} */

/* execute_query(
    r"
    CREATE TABLE IF NOT EXISTS shows (
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

let client = client_connection();
if client.is_some() {
    let error = client.unwrap().execute(
        r"INSERT INTO content (full_path) VALUES ($1)",
        &[&content.get_full_path()],
    );
} */
//template

fn execute_query(query: &str) {
    let client = client_connection();
    if client.is_some() {
        let mut client = client.unwrap();
        let error = client.batch_execute(query);
        if error.is_err() {
            print(
                Verbosity::ERROR,
                From::DB,
                "execute_query",
                format!("{}", error.unwrap_err()),
            );
        }
    }
}

pub fn db_ensure_season_exists_in_show(show_uid: usize, season_number: usize) {
    ensure_table_exists();
    if !season_exists_in_show(show_uid, season_number) {
        insert_season(show_uid, season_number);
    }

    fn ensure_table_exists() {
        execute_query(
            r"
            CREATE TABLE IF NOT EXISTS season (
                show_uid             INTEGER REFERENCES show (show_uid) NOT NULL,
                season_number        SMALLINT NOT NULL,
				PRIMARY KEY (show_uid, season_number)
            )",
        );
    }

    fn insert_season(show_uid: usize, season_number: usize) {
        let show_uid = show_uid as i32;
        let season_number = season_number as i16;
        let client = client_connection();
        if client.is_some() {
            let error = client.unwrap().execute(
                r"INSERT INTO season (show_uid, season_number) VALUES ($1, $2)",
                &[&show_uid, &season_number],
            );
            output_insert_error(error, "insert_season");
        }
    }

    fn season_exists_in_show(show_uid: usize, season_number: usize) -> bool {
        let show_uid = show_uid as i32;
        let season_number = season_number as i16;
        let client = client_connection();
        if client.is_some() {
            let result = client.unwrap().query(
                r"SELECT EXISTS(SELECT 1 FROM season WHERE show_uid = $1 AND season_number = $2)",
                &[&show_uid, &season_number],
            );
            if result.is_ok() {
                let result = result.unwrap();
                if result.len() > 0 {
                    let exists: bool = result[0].get(0);
                    if exists {
                        return true;
                    } else {
                        return false;
                    }
                } else {
                    panic!("no rows exist in result")
                }
            } else {
                output_retrieve_error(result, "season_exists_in_show");
                panic!("something is wrong with the returned result")
            }
        } else {
            panic!("client couldn't establish connection");
        }
    }
}

/* pub struct Show {
    pub uid: usize,
    pub title: String,
    pub seasons: Vec<Season>,
} */

pub fn db_get_show_uid_by_title(show_title: String) -> Option<usize> {
    let client = client_connection();
    if client.is_some() {
        let result = client.unwrap().query(
            r"SELECT show_uid from show WHERE title = $1",
            &[&show_title],
        );
        if result.is_ok() {
            let result = result.unwrap();
            let mut uid: Option<i32> = None;
            if result.len() > 0 {
                for row in &result {
                    uid = row.get(0);
                }
                if uid.is_some() {
                    return Some(uid.unwrap() as usize);
                }
            }
        }
    }
    return None;
}

pub fn db_ensure_show_exists(show_title: String) -> Option<usize> {
    ensure_table_exists();

    if !show_exists(&show_title) {
        let qrid = generate_qrid();
        insert_show(show_title, qrid);
        let uid = read_back_show_uid(qrid);
        wipe_show_qrid(qrid);
        return Some(uid);
    } else {
        return db_get_show_uid_by_title(show_title);
    }

    fn ensure_table_exists() {
        execute_query(
            r"
            CREATE TABLE IF NOT EXISTS show (
                show_uid             SERIAL PRIMARY KEY,
                title           TEXT NOT NULL,
                qrid            INTEGER
            )",
        );
    }

    fn insert_show(show_title: String, qrid: i32) {
        let client = client_connection();
        if client.is_some() {
            let error = client.unwrap().execute(
                r"INSERT INTO show (title, qrid) VALUES ($1, $2)",
                &[&show_title, &qrid],
            );
            //use if I need to do anything more with the row
            //let show_uid = read_back_show_uid(qrid);
            output_insert_error(error, "insert_show");
        }
    }

    fn show_exists(show_title: &str) -> bool {
        let client = client_connection();
        if client.is_some() {
            let result = client.unwrap().query(
                r"SELECT EXISTS(SELECT 1 FROM show WHERE title = $1)",
                &[&show_title],
            );
            if result.is_ok() {
                let result = result.unwrap();
                if result.len() > 0 {
                    let exists: bool = result[0].get(0);
                    if exists {
                        return true;
                    } else {
                        return false;
                    }
                }
            }
        } else {
            panic!("client couldn't establish connection");
        }
        panic!("show_exists should have returned a boolean regardless, this shouldn't happen!");
    }

    fn read_back_show_uid(qrid: i32) -> usize {
        let client = client_connection();
        if client.is_some() {
            //i only want the latest one that fits this query to contend with the potential statistical crossover
            let result = client
                .unwrap()
                .query(r"SELECT show_uid from show WHERE qrid = $1", &[&qrid]);
            if result.is_ok() {
                let result = result.unwrap();
                let mut uid: Option<i32> = None;
                if result.len() > 0 {
                    for row in &result {
                        uid = row.get(0);
                    }
                    if uid.is_some() {
                        return uid.unwrap() as usize;
                    }
                }
            }
        }
        panic!("Couldn't find entry that was just inserted, this shouldn't happen.");
    }

    fn wipe_show_qrid(qrid: i32) {
        let client = client_connection();
        if client.is_some() {
            let error = client
                .unwrap()
                .execute(r"UPDATE show SET qrid = NULL WHERE qrid = $1", &[&qrid]);
            output_insert_error(error, "wipe_show_qrid")
        }
    }
}

/* pub struct Content {
    pub uid: usize,
    pub full_path: PathBuf,
    pub designation: Designation,
    //pub job_queue: VecDeque<Job>,
    pub hash: Option<u64>,
    //pub versions: Vec<FileVersion>,
    //pub metadata_dump
    pub show_uid: Option<usize>,
    pub show_title: Option<String>,
    pub show_season_episode: Option<(usize, usize)>,
}
 */
//i want the auto generated ID of the entry
pub fn db_insert_content(content: Content) {
    ensure_table_exists();
    insert_content(content.clone());
    if content.designation == crate::designation::Designation::Episode {
        let show_uid = db_ensure_show_exists(content.show_title.unwrap());
        if show_uid.is_some() {
            db_ensure_season_exists_in_show(
                show_uid.unwrap(),
                content.show_season_episode.unwrap().0,
            );
        } else {
            panic!("show UID couldn't be retreived")
        }
    }

    fn ensure_table_exists() {
        execute_query(
            r"
            CREATE TABLE IF NOT EXISTS content (
                uid             SERIAL PRIMARY KEY,
                full_path       TEXT NOT NULL
            )",
        );
    }
    fn insert_content(content: Content) {
        let client = client_connection();
        if client.is_some() {
            let error = client.unwrap().execute(
                r"INSERT INTO content (full_path) VALUES ($1)",
                &[&content.get_full_path()],
            );
            output_insert_error(error, "insert_content");
        }
    }
}

pub fn db_insert_task(task_id: usize, id: usize, job_uid: usize) {
    ensure_table_exists();
    insert_task(task_id, id, job_uid);

    //pull out in order by id
    fn ensure_table_exists() {
        execute_query(
            r"
            CREATE TABLE IF NOT EXISTS job_task_queue (
                id                  INTEGER NOT NULL,
                job_uid             INTEGER REFERENCES job_queue (job_uid) NOT NULL,
                task_id             SMALLINT NOT NULL,
                PRIMARY KEY(job_uid, id)
            );",
        );
    }

    fn insert_task(task_id: usize, id: usize, job_uid: usize) {
        let client = client_connection();
        if client.is_some() {
            let id = id as i32;
            let job_uid = job_uid as i32;
            //hopefully won't overflow, but I doubt it ever will, it would require 32k unique tasks
            let task_id = task_id as i16;

            let error = client.unwrap().execute(
                r"INSERT INTO job_task_queue (
                        id,
                        job_uid,
                        task_id
                    ) VALUES ($1, $2, $3)",
                &[&id, &job_uid, &task_id],
            );
            output_insert_error(error, "insert_task");
            print(
                Verbosity::INFO,
                From::DB,
                "db_insert_task",
                format!("[job_uid: {}][id: {}][task_id: {}]", job_uid, id, task_id),
            );
        }
    }
}

pub fn db_insert_job(job: Job) {
    ensure_table_exists();
    insert_job(job);

    fn ensure_table_exists() {
        //ensures job table exists
        //cache_directory marked as not null, but realistically it can be None, but won't be shown as such in the database,
        //it provides no benefit and something else will be stored in the database designate no usable value
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
                worker_string_id    TEXT NOT NULL,
                qrid                INTEGER NOT NULL
            )",
        );
    }

    fn db_read_back_job_uid(qrid: i32) -> usize {
        let client = client_connection();
        if client.is_some() {
            //i only want the latest one that fits this query to contend with the potential statistical crossover
            let result = client
                .unwrap()
                .query(r"SELECT job_uid from job_queue WHERE qrid = $1", &[&qrid]);
            if result.is_ok() {
                let result = result.unwrap();
                let mut uid: Option<i32> = None;
                if result.len() > 0 {
                    for row in &result {
                        uid = row.get(0);
                    }
                    if uid.is_some() {
                        return uid.unwrap() as usize;
                    }
                }
            }
        }
        panic!("Couldn't find entry that was just inserted, this shouldn't happen.");
    }

    fn insert_job(job: Job) {
        //get client and inserts job if the client connection is fine
        let client = client_connection();
        if client.is_some() {
            //quick retrieve ID generation
            let qrid = generate_qrid();

            let worker_uid = job.worker.clone().unwrap().0 as i32;
            let worker_string_identifier = job.worker.unwrap().1;

            let error = client.unwrap().execute(
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
            );
            output_insert_error(error, "insert_job");
            let uid = db_read_back_job_uid(qrid);
            print(
                Verbosity::INFO,
                From::DB,
                "insert_job",
                format!(
                    "[job_uid: {}][Source: {}][Encode: {}]",
                    uid,
                    job.source_path.to_string_lossy().to_string(),
                    job.encode_path.to_string_lossy().to_string()
                ),
            );
            for (pos, task) in job.tasks.iter().enumerate() {
                db_insert_task(task.clone() as usize, pos, uid);
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
    //the order for dropping tables matters if foreign keys exist (job_task_queue has a foreign key of job_queue)
    let tables: Vec<&str> = vec!["content", "job_task_queue", "job_queue", "season", "show"];
    for table in tables {
        execute_query(&format!("DROP TABLE IF EXISTS {}", table))
    }
}

pub fn print_jobs() {
    let result = db_get_by_query(r"SELECT uid FROM job_queue");
    if result.is_some() {
        let result = result.unwrap();
        if result.is_ok() {
            let result = result.unwrap();
            for row in result {
                let uid: i32 = row.get(0);
                print(
                    Verbosity::INFO,
                    From::DB,
                    "print_jobs",
                    format!("[uid: {}]", uid),
                )
            }
        }
    }
}

pub fn print_shows() {
    let result = db_get_by_query(r"SELECT title FROM show");
    if result.is_some() {
        let result = result.unwrap();
        if result.is_ok() {
            let result = result.unwrap();
            for row in result {
                let title: String = row.get(0);
                print(
                    Verbosity::INFO,
                    From::DB,
                    "print_shows",
                    format!("[title: {}]", title),
                )
            }
        }
    }
}

pub fn print_seasons() {
    let result = db_get_by_query(r"SELECT show_uid, season_number FROM season");
    if result.is_some() {
        let result = result.unwrap();
        if result.is_ok() {
            let result = result.unwrap();
            for row in result {
                let show_uid: i32 = row.get(0);
                let season_number: i16 = row.get(1);
                print(
                    Verbosity::INFO,
                    From::DB,
                    "print_seasons",
                    format!("[show_uid: {}][season_number: {}]", show_uid, season_number),
                )
            }
        }
    }
}

pub fn print_contents() {
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
                print(
                    Verbosity::DEBUG,
                    From::DB,
                    "print_content",
                    format!("{:3}:{}", uid, full_path_temp),
                )
            }
        }
    }
}
