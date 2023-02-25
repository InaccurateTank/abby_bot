CREATE TABLE IF NOT EXISTS servers
(
    id      TEXT    PRIMARY KEY NOT NULL,
    memes   BOOL                NOT NULL,
    respond BOOL                NOT NULL,
    roles   TEXT
);
