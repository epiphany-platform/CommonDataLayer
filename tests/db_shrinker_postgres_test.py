import json
import os
import unittest

import psycopg2

def load_case(case_name) -> tuple:
    with open(os.path.join(os.path.dirname(os.path.realpath(__file__)), 'data', 'db_shrinker_postgres', f'{case_name}.json')) as file:
        json_document = json.load(file)
        return json_document['data'], json_document['expected']


class DbShrinkerPostgresTests(unittest.TestCase):
    def setUp(self):
        self.psql_url = os.getenv("POSTGRES_CONNECTION_URL") or "postgresql://postgres:1234@localhost:5432/postgres"
        self.shrinker = os.getenv("DB_SHRINKER_POSTGRES_EXE") or "db-shrinker-postgres"
        self.db = psycopg2.connect(self.psql_url)

    def tearDown(self):
        curr = self.db.cursor()
        curr.execute("DELETE FROM data WHERE true")
        self.db.commit()
        curr.close()
        self.db.close()

    def test_shrinking_when_field_added(self):
        data, expected = load_case('field_added')

        self.insert_test_data(data)

        self.run_shrinker()

        rows = self.fetch_results()

        self.assertSequenceEqual(rows, expected)

    def test_shrinking_when_field_deleted(self):
        data, expected = load_case('field_deleted')
        curr = self.db.cursor()

        self.insert_test_data(data)

        self.run_shrinker()

        rows = self.fetch_results()

        self.assertSequenceEqual(rows, expected)

    def test_shrinking_when_partial_update(self):
        data, expected = load_case('partial_update')
        curr = self.db.cursor()

        self.insert_test_data(data)

        self.run_shrinker()

        rows = self.fetch_results()

        self.assertSequenceEqual(rows, expected)

    def test_shrinking_when_simple_override(self):
        data, expected = load_case('simple_override')
        curr = self.db.cursor()

        self.insert_test_data(data)

        self.run_shrinker()

        rows = self.fetch_results()

        self.assertSequenceEqual(rows, expected)

    def fetch_results(self):
        curr = self.db.cursor()
        curr.execute('SELECT * FROM data')
        rows = curr.fetchall()
        rows = [{'object_id': row[0], 'version': row[1], 'schema_id': row[2], 'payload': row[3]} for row in rows]
        curr.close()
        return rows

    def run_shrinker(self):
        os.system(f'{self.shrinker} "{self.psql_url}"')

    def insert_test_data(self, data):
        curr = self.db.cursor()
        for entry in data:
            curr.execute('INSERT INTO data (object_id, version, schema_id, payload) VALUES (%s, %s, %s, %s)',
                         (entry['object_id'], entry['version'], entry['schema_id'], json.dumps(entry['payload'])))
        self.db.commit()


if __name__ == '__main__':
    unittest.main()
