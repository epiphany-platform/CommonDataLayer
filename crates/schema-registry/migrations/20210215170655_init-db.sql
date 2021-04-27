\set ON_ERROR_STOP off
SET search_path to 'public,cdlgrpc,cdlkafka';

-- database specific
DROP EXTENSION "uuid-ossp";
CREATE EXTENSION "uuid-ossp" WITH SCHEMA public;
-- schema specific
DROP SCHEMA cdlkafka CASCADE;
DROP SCHEMA cdlgrpc CASCADE;

-- actual init
CREATE SCHEMA cdlkafka;
CREATE SCHEMA cdlgrpc;
SET search_path to 'cdlgrpc,cdlkafka';

CREATE TABLE IF NOT EXISTS cdlgrpc.data (
    object_id UUID NOT NULL,
    version BIGINT NOT NULL,
    schema_id UUID NOT NULL,
    payload JSON NOT NULL,
    PRIMARY KEY (object_id, version)
);

CREATE TABLE IF NOT EXISTS cdlkafka.data (
    object_id UUID NOT NULL,
    version BIGINT NOT NULL,
    schema_id UUID NOT NULL,
    payload JSON NOT NULL,
    PRIMARY KEY (object_id, version)
);
SET search_path to public,cdlgrpc,cdlkafka;

CREATE TABLE IF NOT EXISTS cdlkafka.relations (
    id UUID NOT NULL UNIQUE DEFAULT uuid_generate_v1(),
    parent_schema_id UUID NOT NULL,
    child_schema_id UUID NOT NULL
);

CREATE TABLE IF NOT EXISTS cdlkafka.edges (
    relation_id UUID NOT NULL,
    parent_object_id UUID NOT NULL,
    child_object_id UUID NOT NULL,
    FOREIGN KEY (relation_id) REFERENCES cdlkafka.relations(id)
);

CREATE TABLE IF NOT EXISTS cdlgrpc.relations (
    id UUID NOT NULL UNIQUE DEFAULT uuid_generate_v1(),
    parent_schema_id UUID NOT NULL,
    child_schema_id UUID NOT NULL
);

CREATE TABLE IF NOT EXISTS cdlgrpc.edges (
    relation_id UUID NOT NULL,
    parent_object_id UUID NOT NULL,
    child_object_id UUID NOT NULL,
    FOREIGN KEY (relation_id) REFERENCES cdlgrpc.relations(id)
);
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
SET SCHEMA 'cdlkafka';
SET search_path to public,cdlkafka;

DROP TABLE IF EXISTS cdlkafka.schemas CASCADE;
DROP TABLE IF EXISTS cdlkafka.views CASCADE;
DROP TABLE IF EXISTS cdlkafka.definitions CASCADE;
DROP TYPE IF EXISTS cdlkafka.schema_type_enum CASCADE ;
DROP TRIGGER IF EXISTS  notify_view_updated ON cdlkafka.views CASCADE ;
DROP TRIGGER IF EXISTS notify_schema_updated ON cdlkafka.schemas CASCADE ;
DROP FUNCTION IF EXISTS cdlkafka.notify_row_updated ( ) CASCADE ;


CREATE TYPE cdlkafka.schema_type_enum AS ENUM ('documentstorage', 'timeseries');

CREATE TABLE cdlkafka.schemas (
    id                 uuid primary key not null,
    name               varchar not null,
    schema_type        schema_type_enum not null,
    insert_destination varchar not null,
    query_address      varchar not null
);

CREATE TABLE cdlkafka.views (
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

CREATE TABLE cdlkafka.definitions (
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
SET SCHEMA 'cdlkafka';
INSERT INTO schemas(id, name, schema_type, insert_destination, query_address) VALUES('2cfad3c7-411a-11eb-8000-000000000000', 'new schema', 'documentstorage', 'http://postgres_command:50202', 'http://postgres_query:50201');
INSERT INTO definitions(version, definition, schema) VALUES('1.0.0', '{"a": "string"}', '2cfad3c7-411a-11eb-8000-000000000000');
INSERT INTO schemas(id, name, schema_type, insert_destination, query_address) VALUES('a5e0c7e2-412c-11eb-8000-000000000000', 'second schema', 'documentstorage', 'http://postgres_command:50202', 'http://postgres_query:50201');
INSERT INTO definitions(version, definition, schema) VALUES('1.0.0', '{"b": "string"}', 'a5e0c7e2-412c-11eb-8000-000000000000');
INSERT INTO views(id, schema, name, materializer_address, materializer_options, fields) VALUES ('ec8cc976-412b-11eb-8000-000000000000', '2cfad3c7-411a-11eb-8000-000000000000', 'new view', 'http://materializer:1234', '{}', '{"foo":{"field_name":"a"}}');
SET SCHEMA 'cdlgrpc';
INSERT INTO schemas(id, name, schema_type, insert_destination, query_address) VALUES('2cfad3c7-411a-11eb-8000-000000000000', 'new schema', 'documentstorage', 'http://postgres_command:50202', 'http://postgres_query:50201');
INSERT INTO definitions(version, definition, schema) VALUES('1.0.0', '{"a": "string"}', '2cfad3c7-411a-11eb-8000-000000000000');
INSERT INTO schemas(id, name, schema_type, insert_destination, query_address) VALUES('a5e0c7e2-412c-11eb-8000-000000000000', 'second schema', 'documentstorage', 'http://postgres_command:50202', 'http://postgres_query:50201');
INSERT INTO definitions(version, definition, schema) VALUES('1.0.0', '{"b": "string"}', 'a5e0c7e2-412c-11eb-8000-000000000000');
INSERT INTO views(id, schema, name, materializer_address, materializer_options, fields) VALUES ('ec8cc976-412b-11eb-8000-000000000000', '2cfad3c7-411a-11eb-8000-000000000000', 'new view', 'http://materializer:1234', '{"table": "MATERIALIZED_VIEW"}', '{"foo":{"field_name":"a"}}');