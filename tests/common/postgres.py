import json

import psycopg2


class PostgresConfig:
    def __init__(self, user='postgres', password='1234', host='localhost', port='5432', dbname='postgres',
                 schema='cdl'):
        self.user = user
        self.password = password
        self.host = host
        self.port = port
        self.dbname = dbname
        self.schema = schema

    def to_dict(self):
        return {
            "POSTGRES_USERNAME": self.user,
            "POSTGRES_PASSWORD": self.password,
            "POSTGRES_HOST": self.host,
            "POSTGRES_PORT": self.port,
            "POSTGRES_DBNAME": self.dbname,
            "POSTGRES_SCHEMA": self.schema,
        }


def connect_to_postgres(config: PostgresConfig):
    return psycopg2.connect(dbname=config.dbname, user=config.user,
                            password=config.password, host=config.host, port=config.port)


def fetch_data(config: PostgresConfig):
    db = connect_to_postgres(config)
    curr = db.cursor()

    curr.execute('SELECT * FROM cdl.data ORDER BY version')
    rows = curr.fetchall()
    rows = [{'object_id': row[0], 'version': row[1],
             'schema_id': row[2], 'payload': row[3]} for row in rows]
    curr.close()
    db.close()
    return rows


def insert_data(config: PostgresConfig, data):
    db = connect_to_postgres(config)
    curr = db.cursor()

    for entry in data:
        curr.execute('INSERT INTO cdl.data (object_id, version, schema_id, payload) VALUES (%s, %s, %s, %s)',
                     (entry['object_id'], entry['version'], entry['schema_id'], json.dumps(entry['payload'])))
    db.commit()
    curr.close()
    db.close()


def clear_data(config: PostgresConfig):
    db = connect_to_postgres(config)
    curr = db.cursor()

    curr.execute('DELETE FROM cdl.data')

    db.commit()
    curr.close()
    db.close()


def clear_relations(config: PostgresConfig):
    db = connect_to_postgres(config)
    curr = db.cursor()

    curr.execute('DELETE FROM cdl.edges')
    curr.execute('DELETE FROM cdl.relations')

    db.commit()
    curr.close()
    db.close()
