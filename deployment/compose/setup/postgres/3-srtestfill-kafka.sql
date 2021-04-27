SET SCHEMA 'cdlkafka';
INSERT INTO schemas(id, name, schema_type, insert_destination, query_address) VALUES('2cfad3c7-411a-11eb-8000-000000000000', 'new schema', 'documentstorage', 'http://postgres_command:50202', 'http://postgres_query:50201');
INSERT INTO definitions(version, definition, schema) VALUES('1.0.0', '{"a": "string"}', '2cfad3c7-411a-11eb-8000-000000000000');
INSERT INTO schemas(id, name, schema_type, insert_destination, query_address) VALUES('a5e0c7e2-412c-11eb-8000-000000000000', 'second schema', 'documentstorage', 'http://postgres_command:50202', 'http://postgres_query:50201');
INSERT INTO definitions(version, definition, schema) VALUES('1.0.0', '{"b": "string"}', 'a5e0c7e2-412c-11eb-8000-000000000000');
INSERT INTO views(id, schema, name, materializer_address, materializer_options, fields) VALUES ('ec8cc976-412b-11eb-8000-000000000000', '2cfad3c7-411a-11eb-8000-000000000000', 'new view', 'http://materializer:1234', '{}', '{"foo":{"field_name":"a"}}');