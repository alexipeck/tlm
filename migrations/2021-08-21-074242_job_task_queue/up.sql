CREATE TABLE IF NOT EXISTS job_task_queue (
    id                  INTEGER NOT NULL,
    job_uid             INTEGER REFERENCES job_queue (job_uid) NOT NULL,
    task_id             SMALLINT NOT NULL,
    PRIMARY KEY(job_uid, id)
)