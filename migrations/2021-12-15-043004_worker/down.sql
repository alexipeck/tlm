drop table worker;

CREATE TABLE IF NOT EXISTS worker (
    id                INTEGER NOT NULL,
    worker_ip_address TEXT NOT NULL,
    PRIMARY KEY(id)
)