SET SCHEMA 'cdlobgrpc';
SET search_path to public,cdlobgrpc;

DROP TABLE IF EXISTS cdlobgrpc.schemas CASCADE;
DROP TABLE IF EXISTS cdlobgrpc.views CASCADE;
DROP TABLE IF EXISTS cdlobgrpc.definitions CASCADE;
DROP TYPE IF EXISTS cdlobgrpc.schema_type_enum CASCADE ;
DROP TRIGGER IF EXISTS  notify_view_updated ON cdlobgrpc.views CASCADE ;
DROP TRIGGER IF EXISTS notify_schema_updated ON cdlobgrpc.schemas CASCADE ;
DROP FUNCTION IF EXISTS cdlobgrpc.notify_row_updated ( ) CASCADE ;


CREATE TYPE cdlobgrpc.schema_type_enum AS ENUM ('documentstorage', 'timeseries');

CREATE TABLE cdlobgrpc.schemas (
    id                 uuid primary key not null,
    name               varchar not null,
    schema_type        schema_type_enum not null,
    insert_destination varchar not null,
    query_address      varchar not null
);

CREATE TABLE cdlobgrpc.views (
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

CREATE TABLE cdlobgrpc.definitions (
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
