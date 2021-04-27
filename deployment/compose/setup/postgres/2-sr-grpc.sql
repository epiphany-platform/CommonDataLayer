SET SCHEMA 'cdlgrpc';
SET search_path to public,cdlgrpc;

DROP TABLE IF EXISTS cdlgrpc.schemas CASCADE;
DROP TABLE IF EXISTS cdlgrpc.views CASCADE;
DROP TABLE IF EXISTS cdlgrpc.definitions CASCADE;
DROP TYPE IF EXISTS cdlgrpc.schema_type_enum CASCADE ;
DROP TRIGGER IF EXISTS  notify_view_updated ON cdlgrpc.views CASCADE ;
DROP TRIGGER IF EXISTS notify_schema_updated ON cdlgrpc.schemas CASCADE ;
DROP FUNCTION IF EXISTS cdlgrpc.notify_row_updated ( ) CASCADE ;


CREATE TYPE cdlgrpc.schema_type_enum AS ENUM ('documentstorage', 'timeseries');

CREATE TABLE cdlgrpc.schemas (
    id                 uuid primary key not null,
    name               varchar not null,
    schema_type        schema_type_enum not null,
    insert_destination varchar not null,
    query_address      varchar not null
);

CREATE TABLE cdlgrpc.views (
    id                   uuid primary key not null,
    name                 varchar not null,
    materializer_address varchar not null,
    materializer_options json not null,
    fields               json not null,
    schema               uuid not null,

    CONSTRAINT fk_schema_1
        FOREIGN KEY(schema)
        REFERENCES schemas(id)
        ON UPDATE CASCADE
        ON DELETE CASCADE
);

CREATE TABLE cdlgrpc.definitions (
    version    varchar not null,
    definition json not null,
    schema     uuid not null,

    PRIMARY KEY(schema, version),
    CONSTRAINT fk_schema_2
        FOREIGN KEY(schema)
        REFERENCES schemas(id)
        ON UPDATE CASCADE
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

CREATE TRIGGER notify_view_updated
    AFTER UPDATE ON views
    FOR EACH ROW
    EXECUTE PROCEDURE notify_row_updated('views');
