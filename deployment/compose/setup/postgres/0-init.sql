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
