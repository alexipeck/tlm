ALTER TABLE content
RENAME TO generic;
CREATE TABLE IF NOT EXISTS generic (
    generic_uid     SERIAL PRIMARY KEY,
    full_path       TEXT NOT NULL,
    designation     INTEGER NOT NULL,
    file_hash       TEXT
)