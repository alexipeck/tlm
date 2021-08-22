CREATE TABLE IF NOT EXISTS content (
    content_uid     SERIAL PRIMARY KEY,
    full_path       TEXT NOT NULL,
    designation     INTEGER NOT NULL
)