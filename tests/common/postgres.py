import json

import psycopg2

from tests.common.config import PostgresConfig


def connect_to_postgres(config: PostgresConfig):
    return psycopg2.connect(dbname=config.dbname, user=config.user,
                            password=config.password, host=config.host, port=config.port)


def fetch_data_table(db):
    curr = db.cursor()

    curr.execute('SELECT * FROM cdl.data ORDER BY version')
    rows = curr.fetchall()
    rows = [{'object_id': row[0], 'version': row[1], 'schema_id': row[2], 'payload': row[3]} for row in rows]
    curr.close()
    return rows


def clear_data_table(db):
    curr = db.cursor()

    curr.execute("DELETE FROM cdl.data WHERE true")
    db.commit()
    curr.close()
    curr.close()


def insert_test_data(db, data):
    curr = db.cursor()

    for entry in data:
        curr.execute('INSERT INTO cdl.data (object_id, version, schema_id, payload) VALUES (%s, %s, %s, %s)',
                     (entry['object_id'], entry['version'], entry['schema_id'], json.dumps(entry['payload'])))
    db.commit()
