import pytest

from tests.common import load_case
from tests.common.cdl_env import cdl_env
from tests.common.config import PostgresConfig
from tests.common.db_shrinker_postgres import DbShrinkerPostgres
from tests.common.postgres import fetch_data_table, insert_test_data, connect_to_postgres


@pytest.fixture(params=['field_added', 'field_deleted', 'partial_update', 'simple_override'])
def shrinking(request):
    with cdl_env('.', postgres_config=PostgresConfig()) as env:
        db = connect_to_postgres(env.postgres_config)
        data, expected = load_case(request.param, 'db_shrinker_postgres')

        insert_test_data(db, data)

        yield db, env.postgres_config, expected

        db.close()


def test_shrinking(shrinking):
    db, postgres_config, expected = shrinking

    DbShrinkerPostgres(postgres_config).run()

    actual = fetch_data_table(db)

    assert actual == expected
