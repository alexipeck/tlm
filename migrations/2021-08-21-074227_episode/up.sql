CREATE TABLE IF NOT EXISTS episode (
    generic_uid             INTEGER REFERENCES generic (generic_uid) NOT NULL,
    show_uid                INTEGER REFERENCES show (show_uid) NOT NULL,
    episode_title           TEXT NOT NULL,
    season_number           SMALLINT NOT NULL,
    episode_number          SMALLINT NOT NULL,
    PRIMARY KEY(generic_uid, show_uid, season_number, episode_number)
)