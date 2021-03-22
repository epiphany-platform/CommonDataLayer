create extension "uuid-ossp";

CREATE TABLE IF NOT EXISTS cdl.relations (
    id UUID NOT NULL UNIQUE DEFAULT uuid_generate_v1(),
    parent_schema_id UUID NOT NULL,
    child_schema_id UUID NOT NULL
);

CREATE TABLE IF NOT EXISTS cdl.edges (
    relation_id UUID NOT NULL,
    parent_object_id UUID NOT NULL,
    child_object_id UUID NOT NULL,
    FOREIGN KEY (relation_id) REFERENCES cdl.relations(id)
);
