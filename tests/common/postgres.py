import json


def fetch_data_table(db):
    curr = db.cursor()
    curr.execute('SELECT * FROM data ORDER BY version')
    rows = curr.fetchall()
    rows = [{'object_id': row[0], 'version': row[1], 'schema_id': row[2], 'payload': row[3]} for row in rows]
    curr.close()
    return rows


def clear_data_table(db):
    curr = db.cursor()
    curr.execute("DELETE FROM data WHERE true")
    db.commit()
    curr.close()
    curr.close()


def insert_test_data(db, data):
    curr = db.cursor()
    for entry in data:
        curr.execute('INSERT INTO data (object_id, version, schema_id, payload) VALUES (%s, %s, %s, %s)',
                     (entry['object_id'], entry['version'], entry['schema_id'], json.dumps(entry['payload'])))
    db.commit()
