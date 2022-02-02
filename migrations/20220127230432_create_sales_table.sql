-- Add migration script here
CREATE TABLE sales
(
    id       uuid        NOT NULL,
    PRIMARY KEY (id),
    prev     TEXT        NOT NULL,
    curr     TEXT        NOT NULL,
    token_id TEXT        NOT NULL,
    date     timestamptz NOT NULL
);