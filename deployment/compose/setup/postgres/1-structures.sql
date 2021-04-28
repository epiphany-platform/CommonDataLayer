SET search_path to public,cdlgrpc,cdlkafka,cdlobgrpc;

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

CREATE TABLE IF NOT EXISTS cdlobgrpc.relations (
    id UUID NOT NULL UNIQUE DEFAULT uuid_generate_v1(),
    parent_schema_id UUID NOT NULL,
    child_schema_id UUID NOT NULL
);

CREATE TABLE IF NOT EXISTS cdlobgrpc.edges (
    relation_id UUID NOT NULL,
    parent_object_id UUID NOT NULL,
    child_object_id UUID NOT NULL,
    FOREIGN KEY (relation_id) REFERENCES cdlobgrpc.relations(id)
);
