-- add migration script here

CREATE TYPE schema_type AS ENUM ('document_storage', 'timeseries');

CREATE TABLE schemas (
    id                 uuid primary key not null,
    name               varchar not null,
    type               schema_type not null,
    insert_destination varchar not null,
    query_address      varchar not null
);

CREATE TABLE definitions (
    version    varchar not null,
    definition json not null,
    schema     uuid not null,

    PRIMARY KEY(schema, version),
    CONSTRAINT fk_schema_1
        FOREIGN KEY(schema)
        REFERENCES schemas(id) 
        ON DELETE CASCADE
);

-- Notify when a row updates
CREATE OR REPLACE FUNCTION notify_row_updated()
    RETURNS trigger AS $$
DECLARE
    channel text := TG_ARGV[0];
BEGIN
    PERFORM pg_notify(
        channel,
        row_to_json(NEW)::text);
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER notify_schema_updated
    AFTER UPDATE ON schemas
    FOR EACH ROW
    EXECUTE PROCEDURE notify_row_updated('schemas');
