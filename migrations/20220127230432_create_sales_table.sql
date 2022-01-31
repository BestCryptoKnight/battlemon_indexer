-- Add migration script here
CREATE TABLE sales
(
    id       uuid        NOT NULL,
    PRIMARY KEY (id),
    "from"   TEXT        NOT NULL,
    "to"     TEXT        NOT NULL,
    token_id TEXT        NOT NULL,
    price    DECIMAL     NOT NULL,
    date     timestamptz NOT NULL
);