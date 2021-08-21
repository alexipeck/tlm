CREATE TABLE IF NOT EXISTS episode (
    content_uid             INTEGER REFERENCES content (content_uid) NOT NULL,
    show_uid                INTEGER REFERENCES show (show_uid) NOT NULL,
    episode_title           TEXT NOT NULL,
    season_number           SMALLINT NOT NULL,
    episode_number          SMALLINT NOT NULL,
    PRIMARY KEY(content_uid, show_uid, season_number, episode_number)
)