use crate::{
    print::{print, From, Verbosity},
    content::Content,
    designation::convert_i32_to_designation,
    job::Job,
    shows::{self, Show},
    task::Task,
    traceback::Traceback,
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

fn get_client(traceback: Traceback) -> Client {
    let mut traceback = traceback.clone();
    traceback.add_location("get_client");

    /*
     * logic
     */
    let connection_string = r"postgresql://localhost:4531/tlmdb?user=postgres&password=786D3JXegfY8uR6shcPB7UF2oVeQf49ynH8vHgn".to_string();
    let client = Client::connect(&connection_string, NoTls);
    match client {
        Err(err) => {
            print(Verbosity::ERROR, From::DB, traceback, format!("client couldn't establish a connection: {}", err.to_string()));
            panic!();
        }
        _ => {
            return client.unwrap();
        }
    }
    //////////
}

fn handle_insert_error(error: Result<u64, Error>, traceback: Traceback) {
    let mut traceback = traceback.clone();
    traceback.add_location("handle_insert_error");

    /*
     * logic
     */
    if error.is_err() {
        print(
            Verbosity::ERROR,
            From::DB,
            traceback,
            format!("{}", error.unwrap_err()),
        );
    }
    //////////
}

fn handle_retrieve_error(error: Result<Vec<Row>, Error>, traceback: Traceback) {
    let mut traceback = traceback.clone();
    traceback.add_location("handle_retrieve_error");

    /*
     * logic
     */
    if error.is_err() {
        print(
            Verbosity::ERROR,
            From::DB,
            traceback.clone(),
            format!("something is wrong with the returned result: {}", error.unwrap_err()),
        );
        panic!();
    }
    //////////
}

fn execute_query(query: &str, traceback: Traceback) {
    let mut traceback = traceback.clone();
    traceback.add_location("execute_query");
    
    /*
     * logic
     */
    let mut client = get_client(traceback.clone());
    let error = client.batch_execute(query);
    if error.is_err() {
        print(Verbosity::ERROR, From::DB, traceback.clone(), format!("{}: {}", String::from(query), error.unwrap_err()));
    }
    //////////
}

fn db_boolean_handle(input: Vec<Row>, traceback: Traceback) -> bool {
    let mut traceback = traceback.clone();
    traceback.add_location("db_boolean_handle");

    /*
     * logic
     */
    if input.len() > 0 {
        let exists: bool = input[0].get(0);
        if exists {
            return true;
        } else {
            return false;
        }
    } else {
        print(Verbosity::CRITICAL, From::DB, traceback, format!("should have returned a boolean from the db, regardless"));
        panic!();
    }
    //////////
}

/* pub struct Content {
    //pub job_queue: VecDeque<Job>,
    pub hash: Option<u64>,
    //pub versions: Vec<FileVersion>,
    //pub metadata_dump
    pub show_uid: Option<usize>,
    pub show_title: Option<String>,
    pub show_season_episode: Option<(usize, usize)>,
} */

pub fn insert_episode_if_episode(
    content: Content,
    traceback: Traceback,
) {
    let mut traceback = traceback.clone();
    traceback.add_location("insert_episode_if_episode");

    /*
     * logic
     */
    if content.content_is_episode(traceback.clone()) {
        ensure_episode_table_exists(traceback.clone());

        //if !season_exists_in_show(show_uid, season_number, traceback.clone()) {
        insert_episode_internal(content, traceback.clone());
        //}
    }
    //////////

    fn ensure_episode_table_exists(traceback: Traceback) {
        let mut traceback = traceback.clone();
        traceback.add_location("ensure_episode_table_exists");

        execute_query(
            r"
            CREATE TABLE IF NOT EXISTS episode (
                episode_uid             SERIAL PRIMARY KEY,
                content_uid             INTEGER REFERENCES content (content_uid) NOT NULL,
                show_uid                INTEGER REFERENCES show (show_uid) NOT NULL,
                episode_title           TEXT NOT NULL,
                season_number           SMALLINT NOT NULL,
                episode_number          SMALLINT NOT NULL
            )",
            traceback,
        );
    }

    fn insert_episode_internal(content: Content, traceback: Traceback) {
        let mut traceback = traceback.clone();
        traceback.add_location("insert_episode_internal");

        /*
         * logic
         */
        let content_uid = 0;
        let show_uid = content.show_uid.unwrap() as i32;
        let episode_title = "";
        let (season_number_temp, episode_number_temp) = content.show_season_episode.unwrap();
        let season_number = season_number_temp as i32;
        let episode_number = episode_number_temp as i32;
        let error = get_client(traceback.clone()).execute(
            r"INSERT INTO episode (content_uid, show_uid, episode_title, episode_number, season_number) VALUES ($1, $2, $3, $4, $5)",
            &[&content_uid, &show_uid, &episode_title, &episode_number, &season_number],
        );
        handle_insert_error(error, traceback.clone());
        //////////
    }

    /* fn season_exists_in_show(
        show_uid: usize,
        season_number: usize,
        traceback: Traceback,
    ) -> bool {
        
        traceback.add_location("season_exists_in_show");

        let show_uid = show_uid as i32;
        let season_number = season_number as i16;
        let mut client = get_client(traceback.clone());
        let result = handle_result_error(
            client.query(
                r"SELECT EXISTS(SELECT 1 FROM season WHERE show_uid = $1 AND season_number = $2)",
                &[&show_uid, &season_number],
            ),
            traceback,
        );
        return db_boolean_handle(result);
    } */
}

fn get_show_uid_by_title(show_title: String, traceback: Traceback) -> Option<usize> {
    let mut traceback = traceback.clone();
    traceback.add_location("get_show_uid_by_title");
    
    /*
     * logic
     */
    let mut client = get_client(traceback.clone());
    let result = handle_result_error(
        client.query(
            r"SELECT show_uid from show WHERE title = $1",
            &[&show_title],
        ),
        traceback,
    );
    let mut uid: Option<i32> = None;
    for row in &result {
        uid = row.get(0);
    }
    if uid.is_some() {
        return Some(uid.unwrap() as usize);
    }
    return None;
    //////////
}

pub fn ensure_show_exists(show_title: String, traceback: Traceback) -> Option<usize> {
    let mut traceback = traceback.clone();
    traceback.add_location("ensure_show_exists");

    /*
     * logic
     */
    ensure_show_table_exists(traceback.clone());
    if !show_exists(&show_title, traceback.clone()) {
        let qrid = generate_qrid();
        insert_show(show_title, qrid, traceback.clone());
        let uid = read_back_show_uid(qrid, traceback.clone());
        wipe_show_qrid(qrid, traceback.clone());
        return Some(uid);
    } else {
        return get_show_uid_by_title(show_title, traceback);
    }
    //////////
    
    fn ensure_show_table_exists(traceback: Traceback) {
        let mut traceback = traceback.clone();
        traceback.add_location("ensure_show_table_exists");

        execute_query(
            r"
            CREATE TABLE IF NOT EXISTS show (
                show_uid        SERIAL PRIMARY KEY,
                title           TEXT NOT NULL,
                qrid            INTEGER
            )",
            traceback,
        );
    }

    fn insert_show(show_title: String, qrid: i32, traceback: Traceback) {
        let mut traceback = traceback.clone();
        traceback.add_location("insert_show");
        
        /*
         * logic
         */
        let mut client = get_client(traceback.clone());
        let error = client.execute(
            r"INSERT INTO show (title, qrid) VALUES ($1, $2)",
            &[&show_title, &qrid],
        );
        //use if I need to do anything more with the row
        //let show_uid = read_back_show_uid(qrid);
        handle_insert_error(error, traceback.clone());
        //////////
    }

    fn show_exists(show_title: &str, traceback: Traceback) -> bool {
        let mut traceback = traceback.clone();
        traceback.add_location("show_exists");

        /*
         * logic
         */
        let mut client = get_client(traceback.clone());
        return db_boolean_handle(handle_result_error(
            client.query(
                r"SELECT EXISTS(SELECT 1 FROM show WHERE title = $1)",
                &[&show_title],
            ),
            traceback.clone(),
        ), traceback);
        //////////
    }

    fn read_back_show_uid(qrid: i32, traceback: Traceback) -> usize {
        let mut traceback = traceback.clone();
        traceback.add_location("read_back_show_uid");

        /*
         * logic
         */
        return get_uid_from_result(handle_result_error(
            get_client(traceback.clone())
                .query(r"SELECT show_uid FROM show WHERE qrid = $1", &[&qrid]),
            traceback.clone(),
        ), traceback);
        //////////
    }

    fn wipe_show_qrid(qrid: i32, traceback: Traceback) {
        let mut traceback = traceback.clone();
        traceback.add_location("wipe_show_qrid");

        /*
         * logic
         */
        let mut client = get_client(traceback.clone());
        let error = client.execute(r"UPDATE show SET qrid = NULL WHERE qrid = $1", &[&qrid]);
        handle_insert_error(error, traceback.clone());
        //////////
    }
}

fn get_uid_from_result(input: Vec<Row>, traceback: Traceback) -> usize {
    let mut traceback = traceback.clone();
    traceback.add_location("get_uid_from_result");

    /*
     * logic
     */
    let mut uid: Option<i32> = None;
    for row in &input {
        uid = row.get(0);
    }
    if uid.is_some() {
        return uid.unwrap() as usize;
    }
    print(Verbosity::CRITICAL, From::DB, traceback, format!("Couldn't find entry that was just inserted, this shouldn't happen."));
    panic!();
    //////////
}

pub fn insert_content(content: Content, traceback: Traceback) {
    let mut traceback = traceback.clone();
    traceback.add_location("insert_content");

    /*
     * logic
     */
    ensure_content_table_exists(traceback.clone());
    let qrid = generate_qrid();
    insert_content_internal(content.clone(), qrid, traceback.clone());
    let uid = read_back_content_uid(qrid, traceback.clone());
    if content.designation == crate::designation::Designation::Episode {
        let show_uid = ensure_show_exists(content.show_title.unwrap(), traceback.clone());
        if show_uid.is_some() {
            /* ensure_season_exists_in_show(
                show_uid.unwrap(),
                content.show_season_episode.unwrap().0,
                traceback.clone(),
            ); */
        } else {
            print(Verbosity::ERROR, From::DB, traceback, format!("show UID couldn't be retrieved"));
            panic!();
        }
    }
    //////////

    fn read_back_content_uid(qrid: i32, traceback: Traceback) -> usize {
        let mut traceback = traceback.clone();
        traceback.add_location("read_back_content_uid");

        return get_uid_from_result(handle_result_error(
            get_client(traceback.clone())
                .query(r"SELECT content_uid FROM content WHERE qrid = $1", &[&qrid]),
            traceback.clone(),
        ), traceback);
    }

    fn ensure_content_table_exists(traceback: Traceback) {
        let mut traceback = traceback.clone();
        traceback.add_location("ensure_content_table_exists");

        execute_query(
            r"
            CREATE TABLE IF NOT EXISTS content (
                content_uid     SERIAL PRIMARY KEY,
                full_path       TEXT NOT NULL,
                designation     INTEGER NOT NULL,
                qrid            INTEGER
            )",
            traceback,
        );
    }

    fn insert_content_internal(content: Content, qrid: i32, traceback: Traceback) {
        let mut traceback = traceback.clone();
        traceback.add_location("insert_content");

        let designation = content.designation as i32;
        handle_insert_error(
            get_client(traceback.clone()).execute(
                r"INSERT INTO content (full_path, designation, qrid) VALUES ($1, $2, $3)",
                &[&content.get_full_path(), &designation, &qrid],
            ),
            traceback.clone(),
        );
    }
}

fn insert_task(task_id: usize, id: usize, job_uid: usize, traceback: Traceback) {
    let mut traceback = traceback.clone();
    traceback.add_location("insert_task");

    /*
     * logic
     */
    ensure_task_table_exists(traceback.clone());
    insert_task_internal(task_id, id, job_uid, traceback.clone());
    //////////

    //pull out in order by id
    fn ensure_task_table_exists(traceback: Traceback) {
        let mut traceback = traceback.clone();
        traceback.add_location("ensure_task_table_exists");
        execute_query(
            r"
            CREATE TABLE IF NOT EXISTS job_task_queue (
                id                  INTEGER NOT NULL,
                job_uid             INTEGER REFERENCES job_queue (job_uid) NOT NULL,
                task_id             SMALLINT NOT NULL,
                PRIMARY KEY(job_uid, id)
            );",
            traceback,
        );
    }

    fn insert_task_internal(task_id: usize, id: usize, job_uid: usize, traceback: Traceback) {
        let mut traceback = traceback.clone();
        traceback.add_location("insert_task_internal");

        /*
         * logic
         */
        let mut client = get_client(traceback.clone());
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
        handle_insert_error(error, traceback.clone());
        print(
            Verbosity::INFO,
            From::DB,
            traceback.clone(),
            format!("[job_uid: {}][id: {}][task_id: {}]", job_uid, id, task_id),
        );
        //////////
    }
}

pub fn insert_job(job: Job, traceback: Traceback) {
    let mut traceback = traceback.clone();
    traceback.add_location("insert_job");

    /*
     * logic
     */
    ensure_job_table_exists(traceback.clone());
    let uid = insert_job_internal(job, traceback.clone());
    //////////

    fn insert_job_internal(job: Job, traceback: Traceback) -> usize {
        let mut traceback = traceback.clone();
        traceback.add_location("insert_job_internal");

        //get client and inserts job if the client connection is fine
        let mut client = get_client(traceback.clone().clone());
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
            traceback.clone(),
        );
        let uid = read_back_job_uid(qrid, traceback.clone());
        print(
            Verbosity::INFO,
            From::DB,
            traceback.clone(),
            format!(
                "[job_uid: {}][Source: {}][Encode: {}]",
                uid,
                job.source_path.to_string_lossy().to_string(),
                job.encode_path.to_string_lossy().to_string()
            ),
        );
        for (pos, task) in job.tasks.iter().enumerate() {
            insert_task(task.clone() as usize, pos, uid, traceback.clone());
        }
        return uid;
    }

    fn ensure_job_table_exists(traceback: Traceback) {
        let mut traceback = traceback.clone();
        traceback.add_location("ensure_job_table_exists");

        /*
         * logic
         */
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
            traceback,
        );
        //////////
    }

    fn read_back_job_uid(qrid: i32, traceback: Traceback) -> usize {
        let mut traceback = traceback.clone();
        traceback.add_location("read_back_job_uid");

        /*
         * logic
         */
        return get_uid_from_result(handle_result_error(
            get_client(traceback.clone())
                .query(r"SELECT job_uid from job_queue WHERE qrid = $1", &[&qrid]),
            traceback.clone(),
        ), traceback);
        //////////
    }
}

fn handle_result_error(result: Result<Vec<Row>, Error>, traceback: Traceback) -> Vec<Row> {
    let mut traceback = traceback.clone();
    traceback.add_location("handle_result_error");

    /*
     * logic
     */
    if result.is_ok() {
        let result = result.unwrap();
        if result.len() > 0 {
            return result;
        } else {
            print(
                Verbosity::ERROR, 
                From::DB, 
                traceback.clone(), 
                format!("CF: {}, result contained no rows",
                traceback.to_string()
            ));
        }
    } else {
        handle_retrieve_error(result, traceback.clone());
    }
    print(Verbosity::ERROR, From::DB, traceback, format!("couldn't or haven't handled the error yet"));
    panic!();
    //////////
}

fn get_by_query(query: &str, traceback: Traceback) -> Vec<Row> {
    let mut traceback = traceback.clone();
    traceback.add_location("get_by_query");

    let result = get_client(traceback.clone()).query(query, &[]);
    return handle_result_error(result, traceback.clone());
}

pub fn db_purge(traceback: Traceback) {
    let mut traceback = traceback.clone();
    traceback.add_location("db_purge");

    //the order for dropping tables matters if foreign keys exist (job_task_queue has a foreign key of job_queue)
    let tables: Vec<&str> = vec!["content", "job_task_queue", "job_queue", "episode", "season", "show"];
    for table in tables {
        execute_query(
            &format!("DROP TABLE IF EXISTS {}", table),
            traceback.clone(),
        )
    }
}

pub fn print_jobs(traceback: Traceback) {
    let mut traceback = traceback.clone();
    traceback.add_location("print_jobs");

    /*
     * logic
     */
    for row in get_by_query(r"SELECT job_uid FROM job_queue", traceback.clone()) {
        let uid: i32 = row.get(0);
        print(
            Verbosity::INFO,
            From::DB,
            traceback.clone(),
            format!("[job_uid: {}]", uid),
        );
    }
    //////////
}

pub fn print_shows(traceback: Traceback) {
    let mut traceback = traceback.clone();
    traceback.add_location("print_shows");

    /*
     * logic
     */
    for row in get_by_query(r"SELECT title FROM show", traceback.clone()) {
        let title: String = row.get(0);
        print(
            Verbosity::INFO,
            From::DB,
            traceback.clone(),
            format!("[title: {}]", title),
        );
    }
    //////////
}

pub fn print_contents(traceback: Traceback) {
    let mut traceback = traceback.clone();
    traceback.add_location("print_contents");

    /*
     * logic
     */
    for row in get_by_query(
        r"SELECT content_uid, full_path, designation FROM content",
        traceback.clone(),
    ) {
        let content_uid_temp: i32 = row.get(0);
        let content_uid = content_uid_temp as usize;
        let full_path_temp: String = row.get(1);
        let designation_temp: i32 = row.get(2);
        let designation = convert_i32_to_designation(designation_temp);
        let full_path = PathBuf::from(&full_path_temp);
        print(
            Verbosity::DEBUG,
            From::DB,
            traceback.clone(),
            format!(
                "[content_uid: {:2}][designation: {}][full_path: {}]",
                content_uid, designation as i32, full_path_temp
            ),
        )
    }
    //////////
}
