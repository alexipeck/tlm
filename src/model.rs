use super::schema::{content, episode};

#[derive(Insertable)]
#[table_name="content"]
pub struct NewContent {
    pub full_path: String,
    pub designation: i32,
}

#[derive(Queryable)]
pub struct ContentModel {
    pub content_uid: i32,
    pub full_path: String,
    pub designation: i32,
}

#[derive(Insertable)]
#[table_name="episode"]
pub struct NewEpisode {
    pub content_uid: i32,
    pub show_uid: i32,
    pub episode_title: String,
    pub season_number: i32,
    pub episode_number: i32,
}

#[derive(Queryable)]
pub struct EpisodeModel {
    content_uid: i32,
    show_uid: i32,
    episode_title: String,
    season_number: i32,
    episode_number: i32,
}

#[derive(Queryable)]
pub struct JobQueueModel{
        job_uid: i32,
        source_path: String,
        encode_path: String,
        cache_directory: String,
        encode_string: String,
        status_underway: bool,
        status_completed: bool,
        worker_uid: i32,
        worker_string_id: String,
}

#[derive(Queryable)]
pub struct JobTaskQueueModel {
    id: i32,
    job_uid: i32,
    task_id: i32,
}

#[derive(Queryable)]
pub struct ShowModel {
    show_uid: i32,
    title: String,
}