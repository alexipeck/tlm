ALTER TABLE generic
DROP COLUMN full_path;
ALTER TABLE generic
DROP COLUMN file_hash;
ALTER TABLE generic
DROP COLUMN fast_file_hash;
ALTER TABLE generic
DROP COLUMN width;
ALTER TABLE generic
DROP COLUMN height;
ALTER TABLE generic
DROP COLUMN framerate;
ALTER TABLE generic
DROP COLUMN length_time;
ALTER TABLE generic
DROP COLUMN resolution_standard;
ALTER TABLE generic
DROP COLUMN container;

CREATE TABLE IF NOT EXISTS file_version (
    id SERIAL PRIMARY KEY,
    generic_uid INTEGER NOT NULL,
    full_path TEXT NOT NULL,
    master_file BOOLEAN NOT NULL,
    file_hash VARCHAR(20),
    fast_file_hash VARCHAR(20),
    width INTEGER,
    height INTEGER,
    framerate FLOAT,
    length_time FLOAT,
    resolution_standard INTEGER,
    container INTEGER,
    FOREIGN KEY (generic_uid) REFERENCES generic(generic_uid)
)