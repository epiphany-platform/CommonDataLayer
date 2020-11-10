import os
import psycopg2
import pytest

from tests.common import load_case
from tests.common.postgres import fetch_data_table, clear_data_table, insert_test_data

PSQL_URL = os.getenv("POSTGRES_CONNECTION_URL") or "postgresql://postgres:1234@localhost:5432/postgres"
EXECUTABLE = os.getenv("DB_SHRINKER_POSTGRES_EXE") or "db-shrinker-postgres"


def run_shrinker():
    os.system(f'{EXECUTABLE} "{PSQL_URL}"')


@pytest.fixture(params=['field_added', 'field_deleted', 'partial_update', 'simple_override'])
def shrinking(request):
    db = psycopg2.connect(PSQL_URL)

    data, expected = load_case(request.param, "db_shrinker_postgres")

    insert_test_data(db, data)

    yield db, expected

    clear_data_table(db)
    db.close()


def test_shrinking(shrinking):
    db, expected = shrinking

    run_shrinker()

    actual = fetch_data_table(db)

    assert actual == expected
