ALTER TABLE generic
ADD COLUMN full_path TEXT NOT NULL;
ADD COLUMN file_hash VARCHAR(20);
ADD COLUMN fast_file_hash VARCHAR(20);
ADD COLUMN width INTEGER;
ADD COLUMN height INTEGER;
ADD COLUMN framerate FLOAT;
ADD COLUMN length_time FLOAT;
ADD COLUMN resolution_standard INTEGER;
ADD COLUMN container INTEGER;

DROP TABLE file_version;