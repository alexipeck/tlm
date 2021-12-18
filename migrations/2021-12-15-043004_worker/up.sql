drop table worker;

CREATE TABLE IF NOT EXISTS worker (
    id                serial,
    worker_ip_address TEXT NOT NULL,
    PRIMARY KEY(id)
)
