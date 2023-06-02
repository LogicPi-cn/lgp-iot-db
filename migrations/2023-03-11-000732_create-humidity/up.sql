-- Your SQL goes here
CREATE TABLE humiture_datas (
    id          SERIAL       PRIMARY KEY,
    sn          VARCHAR(255) NOT NULL,
    device_id   VARCHAR(255) NOT NULL,
    group_id    INTEGER      NOT NULL,
    type_id     INTEGER      NOT NULL,
    ts          TIMESTAMP    NOT NULL,
    temperature REAL         NOT NULL,
    humidity    REAL         NOT NULL
)