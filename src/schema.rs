table! {
    generic (id) {
        id -> Int4,
        full_path -> Text,
        designation -> Int4,
        file_hash -> Nullable<Varchar>,
    }
}

table! {
    episode (generic_uid, show_uid, show_title, episode_title, season_number, episode_number) {
        generic_uid -> Int4,
        show_uid -> Int4,
        show_title -> Text,
        episode_title -> Text,
        season_number -> Int4,
        episode_number -> Int4,
    }
}

table! {
    job_queue (job_uid) {
        job_uid -> Int4,
        source_path -> Text,
        encode_path -> Text,
        cache_directory -> Text,
        encode_string -> Text,
        status_underway -> Bool,
        status_completed -> Bool,
        worker_uid -> Int4,
        worker_string_id -> Text,
    }
}

table! {
    job_task_queue (job_uid, id) {
        id -> Int4,
        job_uid -> Int4,
        task_id -> Int2,
    }
}

table! {
    show (show_uid) {
        show_uid -> Int4,
        title -> Text,
    }
}

joinable!(episode -> generic (generic_uid));
joinable!(episode -> show (show_uid));
joinable!(job_task_queue -> job_queue (job_uid));

allow_tables_to_appear_in_same_query!(generic, episode, job_queue, job_task_queue, show,);
