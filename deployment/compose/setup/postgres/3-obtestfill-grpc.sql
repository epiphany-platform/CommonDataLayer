SET SCHEMA 'cdlkafka';

INSERT INTO schemas(id, name, schema_type, insert_destination, query_address) VALUES('2cfad3c7-411a-11eb-8000-000000000000', 'new schema', 'documentstorage', 'cdl.document.data', 'http://localhost:50102');
INSERT INTO definitions(version, definition, schema) VALUES('1.0.0', '{"version": "1.0.0","definition": {"a": "string"}}', '2cfad3c7-411a-11eb-8000-000000000000');
INSERT INTO views(id, schema, name, materializer_address, materializer_options, fields) VALUES ('ec8cc976-412b-11eb-8000-000000000000', '2cfad3c7-411a-11eb-8000-000000000000', 'new view', 'http://localhost:1234', '{}', '{"foo": {"field_name": "name"},"department": {"field_name": "department"}}');

INSERT INTO schemas(id, name, schema_type, insert_destination, query_address) VALUES('a5e0c7e2-412c-11eb-8000-000000000000', 'second schema', 'documentstorage', 'cdl.document.data', 'http://localhost:50102');
INSERT INTO definitions(version, definition, schema) VALUES('1.0.0', '{"name": "string","department": "string"}', 'a5e0c7e2-412c-11eb-8000-000000000000');