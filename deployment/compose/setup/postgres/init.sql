CREATE SCHEMA cdl;

CREATE TABLE IF NOT EXISTS cdl.data (
    object_id UUID NOT NULL,
    version BIGINT NOT NULL,
    schema_id UUID NOT NULL,
    payload JSON NOT NULL,
    PRIMARY KEY (object_id, version)
);
