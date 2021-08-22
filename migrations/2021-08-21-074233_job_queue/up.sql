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
)