table! {
    episode (generic_uid, show_uid, season_number, episode_number) {
        generic_uid -> Int4,
        show_uid -> Int4,
        episode_title -> Text,
        season_number -> Int4,
        episode_number -> Int4,
    }
}

table! {
    file_version (id) {
        id -> Int4,
        generic_uid -> Int4,
        full_path -> Text,
        master_file -> Bool,
        file_hash -> Nullable<Varchar>,
        fast_file_hash -> Nullable<Varchar>,
        width -> Nullable<Int4>,
        height -> Nullable<Int4>,
        framerate -> Nullable<Float8>,
        length_time -> Nullable<Float8>,
        resolution_standard -> Nullable<Int4>,
        container -> Nullable<Int4>,
    }
}

table! {
    generic (generic_uid) {
        generic_uid -> Int4,
        designation -> Int4,
    }
}

table! {
    show (show_uid) {
        show_uid -> Int4,
        show_title -> Text,
    }
}

table! {
    worker (id) {
        id -> Int4,
        worker_ip_address -> Text,
    }
}

joinable!(episode -> generic (generic_uid));
joinable!(episode -> show (show_uid));
joinable!(file_version -> generic (generic_uid));

allow_tables_to_appear_in_same_query!(
    episode,
    file_version,
    generic,
    show,
    worker,
);
