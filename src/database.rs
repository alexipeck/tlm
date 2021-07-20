use crate::print::{print, Verbosity, From, convert_function_callback_to_string};
use crate::{
    content::{Content, Job, Task},
    shows::{self, Show},
    designation::convert_i32_to_designation,
};
use core::panic;
use postgres::Client;
use rand::Rng;
use std::path::PathBuf;
use tokio_postgres::{Error, NoTls, Row};

//primary helper functions
fn generate_qrid() -> i32 {
    let mut rng = rand::thread_rng();
    let qrid_temp: u32 = rng.gen_range(0..2147483646);
    return qrid_temp as i32;
}

fn get_client(called_from: Vec<&str>) -> Client {
    let mut called_from = called_from.clone();
    called_from.push("get_client");
    let connection_string = r"postgresql://localhost:4531/tlmdb?user=postgres&password=786D3JXegfY8uR6shcPB7UF2oVeQf49ynH8vHgn".to_string();
    let mut client = Client::connect(&connection_string, NoTls);
    match client {
        Err(err) => {
            print(Verbosity::ERROR, From::DB, called_from, err.to_string());
            panic!("client couldn't establish a connection");
        }
        _ => {
            return client.unwrap();
        }
    }
}

fn output_insert_error(error: Result<u64, Error>, called_from: Vec<&str>) {
    if error.is_err() {
        print(
            Verbosity::ERROR,
            From::DB,
            called_from,
            format!("{}", error.unwrap_err()),
        );
    }
}

fn output_retrieve_error(error: Result<Vec<Row>, Error>, called_from: Vec<&str>) {
    if error.is_err() {
        print(
            Verbosity::ERROR,
            From::DB,
            called_from.clone(),
            format!("{}", error.unwrap_err()),
        );
        panic!(&format!("CF: {}, something is wrong with the returned result", convert_function_callback_to_string(called_from.clone())))
    }
}

fn execute_query(query: &str, called_from: Vec<&str>) {
    let mut called_from = called_from.clone();
    called_from.push("execute_query");
    let mut client = get_client(called_from.clone());
    let error = client.batch_execute(query);
    if error.is_err() {
        print(
            Verbosity::ERROR,
            From::DB,
            called_from,
            format!("{}", error.unwrap_err()),
        );
    }
}

fn db_boolean_handle(input: Vec<Row>) -> bool {
    if input.len() > 0 {
        let exists: bool = input[0].get(0);
        if exists {
            return true;
        } else {
            return false;
        }
    } else {
        panic!("should have returned a boolean from the db, regardless")
    }
}

pub fn ensure_season_exists_in_show(show_uid: usize, season_number: usize, called_from: Vec<&str>) {
    let mut called_from = called_from.clone();
    called_from.push("ensure_season_exists_in_show");
    ensure_table_exists(called_from.clone());
    if !season_exists_in_show(show_uid, season_number, called_from.clone()) {
        insert_season(show_uid, season_number, called_from.clone());
    }

    fn ensure_table_exists(called_from: Vec<&str>) {
        let mut called_from = called_from.clone();
        called_from.push("ensure_table_exists");
        execute_query(
            r"
            CREATE TABLE IF NOT EXISTS season (
                show_uid             INTEGER REFERENCES show (show_uid) NOT NULL,
                season_number        SMALLINT NOT NULL,
				PRIMARY KEY (show_uid, season_number)
            )", called_from);
    }

    fn insert_season(show_uid: usize, season_number: usize, called_from: Vec<&str>) {
        let mut called_from = called_from.clone();
        called_from.push("insert_season");
        let show_uid = show_uid as i32;
        let season_number = season_number as i16;
        let mut client = get_client(called_from.clone());
        let error = client.execute(
            r"INSERT INTO season (show_uid, season_number) VALUES ($1, $2)",
            &[&show_uid, &season_number],
        );
        output_insert_error(error, called_from.clone());
    }

    fn season_exists_in_show(show_uid: usize, season_number: usize, called_from: Vec<&str>) -> bool {
        let mut called_from = called_from.clone();
        called_from.push("season_exists_in_show");
        let show_uid = show_uid as i32;
        let season_number = season_number as i16;
        let mut client = get_client(called_from.clone());
        let result = handle_result_error(client.query(
            r"SELECT EXISTS(SELECT 1 FROM season WHERE show_uid = $1 AND season_number = $2)",
            &[&show_uid, &season_number],
        ), called_from);
        return db_boolean_handle(result);
    }
}

pub fn get_show_uid_by_title(show_title: String, called_from: Vec<&str>) -> Option<usize> {
    let mut called_from = called_from.clone();
    called_from.push("get_show_uid_by_title");
    let mut client = get_client(called_from.clone());
    let result = handle_result_error(client.query(
        r"SELECT show_uid from show WHERE title = $1",
        &[&show_title],
    ), called_from);
    let mut uid: Option<i32> = None;
    for row in &result {
        uid = row.get(0);
    }
    if uid.is_some() {
        return Some(uid.unwrap() as usize);
    }
    return None;
}

pub fn ensure_show_exists(show_title: String, called_from: Vec<&str>) -> Option<usize> {
    let mut called_from = called_from.clone();
    called_from.push("ensure_show_exists");
    ensure_table_exists(called_from.clone());

    if !show_exists(&show_title, called_from.clone()) {
        let qrid = generate_qrid();
        insert_show(show_title, qrid, called_from.clone());
        let uid = read_back_show_uid(qrid, called_from.clone());
        wipe_show_qrid(qrid, called_from.clone());
        return Some(uid);
    } else {
        return get_show_uid_by_title(show_title, called_from);
    }

    fn ensure_table_exists(called_from: Vec<&str>) {
        let mut called_from = called_from.clone();
        called_from.push("ensure_table_exists");
        execute_query(
            r"
            CREATE TABLE IF NOT EXISTS show (
                show_uid             SERIAL PRIMARY KEY,
                title           TEXT NOT NULL,
                qrid            INTEGER
            )", called_from);
    }

    fn insert_show(show_title: String, qrid: i32, called_from: Vec<&str>) {
        let mut called_from = called_from.clone();
        called_from.push("insert_show");
        let mut client = get_client(called_from.clone());
        let error = client.execute(
            r"INSERT INTO show (title, qrid) VALUES ($1, $2)",
            &[&show_title, &qrid],
        );
        //use if I need to do anything more with the row
        //let show_uid = read_back_show_uid(qrid);
        output_insert_error(error, called_from.clone());
    }

    fn show_exists(show_title: &str, called_from: Vec<&str>) -> bool {
        let mut called_from = called_from.clone();
        called_from.push("show_exists");
        let mut client = get_client(called_from.clone());
        return db_boolean_handle(handle_result_error(client.query(
            r"SELECT EXISTS(SELECT 1 FROM show WHERE title = $1)",
            &[&show_title],
        ), called_from));
    }

    fn read_back_show_uid(qrid: i32, called_from: Vec<&str>) -> usize {     
        let mut called_from = called_from.clone();
        called_from.push("read_back_show_uid");   
        return get_uid_from_result(handle_result_error(get_client(called_from.clone()).query(r"SELECT show_uid FROM show WHERE qrid = $1", &[&qrid]), called_from));
    }

    fn wipe_show_qrid(qrid: i32, called_from: Vec<&str>) {
        let mut called_from = called_from.clone();
        called_from.push("wipe_show_qrid");
        let mut client = get_client(called_from.clone());
        let error = client.execute(r"UPDATE show SET qrid = NULL WHERE qrid = $1", &[&qrid]);
        output_insert_error(error, called_from.clone());
    }
}

fn get_uid_from_result(input: Vec<Row>) -> usize {
    let mut uid: Option<i32> = None;
    for row in &input {
        uid = row.get(0);
    }
    if uid.is_some() {
        return uid.unwrap() as usize;
    }
    panic!("Couldn't find entry that was just inserted, this shouldn't happen.");
}

pub fn insert_content(content: Content, called_from: Vec<&str>) {
    let mut called_from = called_from.clone();
    called_from.push("insert_content");
    ensure_table_exists(called_from.clone());
    let qrid = generate_qrid();
    insert_content_internal(content.clone(), qrid, called_from.clone());
    let uid = read_back_content_uid(qrid, called_from.clone());
    if content.designation == crate::designation::Designation::Episode {
        let show_uid = ensure_show_exists(content.show_title.unwrap(), called_from.clone());
        if show_uid.is_some() {
            ensure_season_exists_in_show(
                show_uid.unwrap(),
                content.show_season_episode.unwrap().0,
                called_from.clone(),
            );
        } else {
            panic!("show UID couldn't be retreived");
        }
    }

    fn read_back_content_uid(qrid: i32, called_from: Vec<&str>) -> usize {
        let mut called_from = called_from.clone();
        called_from.push("read_back_content_uid");
        return get_uid_from_result(handle_result_error(get_client(called_from.clone()).query(r"SELECT content_uid FROM content WHERE qrid = $1", &[&qrid]), called_from));
    }

    /* pub struct Content {
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

    fn ensure_table_exists(called_from: Vec<&str>) {
        let mut called_from = called_from.clone();
        called_from.push("ensure_table_exists");
        execute_query(
            r"
            CREATE TABLE IF NOT EXISTS content (
                content_uid     SERIAL PRIMARY KEY,
                full_path       TEXT NOT NULL,
                designation     INTEGER NOT NULL,
                qrid            INTEGER
            )",
            called_from,
        );
    }
    
    fn insert_content_internal(content: Content, qrid: i32, called_from: Vec<&str>) {
        let mut called_from = called_from.clone();
        called_from.push("insert_content");
        let designation = content.designation as i32;
        output_insert_error(get_client(called_from.clone()).execute(
            r"INSERT INTO content (full_path, designation, qrid) VALUES ($1, $2, $3)",
            &[&content.get_full_path(), &designation, &qrid],
        ), called_from.clone());
    }
}

pub fn insert_task(task_id: usize, id: usize, job_uid: usize, called_from: Vec<&str>) {
    let mut called_from = called_from.clone();
    called_from.push("insert_task");
    ensure_table_exists(called_from.clone());
    insert_task_internal(task_id, id, job_uid,called_from.clone());

    //pull out in order by id
    fn ensure_table_exists(called_from: Vec<&str>) {
        let mut called_from = called_from.clone();
        called_from.push("ensure_table_exists");
        execute_query(
            r"
            CREATE TABLE IF NOT EXISTS job_task_queue (
                id                  INTEGER NOT NULL,
                job_uid             INTEGER REFERENCES job_queue (job_uid) NOT NULL,
                task_id             SMALLINT NOT NULL,
                PRIMARY KEY(job_uid, id)
            );",
            called_from,
        );
    }

    fn insert_task_internal(task_id: usize, id: usize, job_uid: usize, called_from: Vec<&str>) {
        let mut called_from = called_from.clone();
        called_from.push("insert_task_internal");
        let mut client = get_client(called_from.clone());
        let id = id as i32;
        let job_uid = job_uid as i32;
        //hopefully won't overflow, but I doubt it ever will, it would require 32k unique tasks
        let task_id = task_id as i16;

        let error = client.execute(
            r"INSERT INTO job_task_queue (
                    id,
                    job_uid,
                    task_id
                ) VALUES ($1, $2, $3)",
            &[&id, &job_uid, &task_id],
        );
        output_insert_error(error, called_from.clone());
        print(
            Verbosity::INFO,
            From::DB,
            called_from.clone(),
            format!("[job_uid: {}][id: {}][task_id: {}]", job_uid, id, task_id),
        );
    }
}

pub fn insert_job(job: Job, called_from: Vec<&str>) {
    let mut called_from = called_from.clone();
    called_from.push("insert_job");
    ensure_table_exists(called_from.clone());
    let uid = insert_job_internal(job, called_from.clone());

    fn insert_job_internal(job: Job, called_from: Vec<&str>) -> usize {
        let mut called_from = called_from.clone();
        called_from.push("insert_job_internal");
        //get client and inserts job if the client connection is fine
        let mut client = get_client(called_from.clone().clone());
        //quick retrieve ID
        let qrid = generate_qrid();

        let worker_uid = job.worker.clone().unwrap().0 as i32;
        let worker_string_identifier = job.worker.unwrap().1;

        output_insert_error(client.execute(
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
        ), called_from.clone());
        let uid = read_back_job_uid(qrid, called_from.clone());
        print(
            Verbosity::INFO,
            From::DB,
            called_from.clone(),
            format!(
                "[job_uid: {}][Source: {}][Encode: {}]",
                uid,
                job.source_path.to_string_lossy().to_string(),
                job.encode_path.to_string_lossy().to_string()
            ),
        );
        for (pos, task) in job.tasks.iter().enumerate() {
            insert_task(task.clone() as usize, pos, uid, called_from.clone());
        }
        return uid;
    }

    fn ensure_table_exists(called_from: Vec<&str>) {
        let mut called_from = called_from.clone();
        called_from.push("ensure_table_exists");
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
            called_from,
        );
    }

    fn read_back_job_uid(qrid: i32, called_from: Vec<&str>) -> usize {
        let mut called_from = called_from.clone();
        called_from.push("read_back_job_uid");
        return get_uid_from_result(handle_result_error(get_client(called_from.clone()).query(r"SELECT job_uid from job_queue WHERE qrid = $1", &[&qrid]), called_from));
    }
}

fn handle_result_error(result: Result<Vec<Row>, Error>, called_from: Vec<&str>) -> Vec<Row> {
    let mut called_from = called_from.clone();
    called_from.push("handle_result_error");
    if result.is_ok() {
        let result = result.unwrap();
        if result.len() > 0 {
            return result;
        } else {
            panic!(&format!("CF: {}, result contained no rows", convert_function_callback_to_string(called_from)));
        }
    } else {
        output_retrieve_error(result, called_from);
    }
    panic!("couldn't or haven't handled the error yet");
}

pub fn get_by_query(query: &str, called_from: Vec<&str>) -> Vec<Row> {
    let mut called_from = called_from.clone();
    called_from.push("get_by_query");
    let result = get_client(called_from.clone()).query(query, &[]);
    return handle_result_error(result, called_from.clone());
}

pub fn db_purge(called_from: Vec<&str>) {
    let mut called_from = called_from.clone();
    called_from.push("db_purge");
    //the order for dropping tables matters if foreign keys exist (job_task_queue has a foreign key of job_queue)
    let tables: Vec<&str> = vec!["content", "job_task_queue", "job_queue", "season", "show"];
    for table in tables {
        execute_query(&format!("DROP TABLE IF EXISTS {}", table), called_from.clone())
    }
}

pub fn print_jobs(called_from: Vec<&str>) {
    let mut called_from = called_from.clone();
    called_from.push("print_jobs");
    for row in get_by_query(r"SELECT job_uid FROM job_queue", called_from.clone()) {
        let uid: i32 = row.get(0);
        print(
            Verbosity::INFO,
            From::DB,
            called_from.clone(),
            format!("[job_uid: {}]", uid),
        );
    }
}

pub fn print_shows(called_from: Vec<&str>) {
    let mut called_from = called_from.clone();
    called_from.push("print_shows");
    for row in get_by_query(r"SELECT title FROM show", called_from.clone()) {
        let title: String = row.get(0);
        print(
            Verbosity::INFO,
            From::DB,
            called_from.clone(),
            format!("[title: {}]", title),
        );
    }
}

pub fn print_seasons(called_from: Vec<&str>) {
    let mut called_from = called_from.clone();
    called_from.push("print_seasons");
    for row in get_by_query(r"SELECT show_uid, season_number FROM season", called_from.clone()) {
        let show_uid: i32 = row.get(0);
        let season_number: i16 = row.get(1);
        print(
            Verbosity::INFO,
            From::DB,
            called_from.clone(),
            format!("[show_uid: {}][season_number: {}]", show_uid, season_number),
        )
    }
}

pub fn print_contents(called_from: Vec<&str>) {
    let mut called_from = called_from.clone();
    called_from.push("print_contents");
    for row in get_by_query(r"SELECT content_uid, full_path, designation FROM content", called_from.clone()) {
        let content_uid_temp: i32 = row.get(0);
        let content_uid = content_uid_temp as usize;
        let full_path_temp: String = row.get(1);
        let designation_temp: i32 = row.get(2);
        let designation = convert_i32_to_designation(designation_temp);
        let full_path = PathBuf::from(&full_path_temp);
        print(
            Verbosity::DEBUG,
            From::DB,
            called_from.clone(),
            format!("[content_uid: {:2}][designation: {}][full_path: {}]", content_uid, designation as i32, full_path_temp),
        )
    }
}
