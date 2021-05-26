import json

import psycopg2


class PostgresConfig:
    def __init__(self,
                 user='postgres',
                 password='1234',
                 host='localhost',
                 port='5432',
                 dbname='postgres',
                 schema='public'):
        self.user = user
        self.password = password
        self.host = host
        self.port = port
        self.dbname = dbname
        self.schema = schema

    def to_dict(self, app):
        if app is None:
            return {
                "POSTGRES_USERNAME": self.user,
                "POSTGRES_PASSWORD": self.password,
                "POSTGRES_HOST": self.host,
                "POSTGRES_PORT": self.port,
                "POSTGRES_DBNAME": self.dbname,
                "POSTGRES_SCHEMA": self.schema,
            }
        else:
            return {
                f"{app}_POSTGRES__USERNAME": self.user,
                f"{app}_POSTGRES__PASSWORD": self.password,
                f"{app}_POSTGRES__HOST": self.host,
                f"{app}_POSTGRES__PORT": self.port,
                f"{app}_POSTGRES__DBNAME": self.dbname,
                f"{app}_POSTGRES__SCHEMA": self.schema,
            }


def connect_to_postgres(config: PostgresConfig):
    return psycopg2.connect(dbname=config.dbname,
                            user=config.user,
                            password=config.password,
                            host=config.host,
                            port=config.port)


def fetch_data(config: PostgresConfig):
    db = connect_to_postgres(config)
    curr = db.cursor()

    curr.execute('SELECT * FROM data ORDER BY version')
    rows = curr.fetchall()
    rows = [{
        'object_id': row[0],
        'version': row[1],
        'schema_id': row[2],
        'payload': row[3]
    } for row in rows]
    curr.close()
    db.close()
    return rows


def insert_data(config: PostgresConfig, data):
    db = connect_to_postgres(config)
    curr = db.cursor()

    for entry in data:
        curr.execute(
            'INSERT INTO data (object_id, version, schema_id, payload) VALUES (%s, %s, %s, %s)',
            (entry['object_id'], entry['version'], entry['schema_id'],
             json.dumps(entry['payload'])))
    db.commit()
    curr.close()
    db.close()


def clear_data(config: PostgresConfig):
    db = connect_to_postgres(config)
    curr = db.cursor()

    curr.execute('DELETE FROM data')

    db.commit()
    curr.close()
    db.close()


def clear_relations(config: PostgresConfig):
    db = connect_to_postgres(config)
    curr = db.cursor()

    curr.execute('DELETE FROM edges')
    curr.execute('DELETE FROM relations')

    db.commit()
    curr.close()
    db.close()
