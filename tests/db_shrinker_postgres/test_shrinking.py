import pytest

from tests.common import load_case
from tests.common.config import PostgresConfig
from tests.common.db_shrinker_postgres import DbShrinkerPostgres
from tests.common.postgres import clear_data, insert_data, fetch_data


@pytest.fixture(params=['field_added', 'field_deleted', 'partial_update', 'simple_override'])
def shrinking(request):
    data, expected = load_case(request.param, 'db_shrinker_postgres')

    postgres_config=PostgresConfig()

    # prepare environment
    clear_data(postgres_config)

    insert_data(postgres_config, data)

    yield postgres_config, expected

    # cleanup environment
    clear_data(postgres_config)


def test_shrinking(shrinking):
    postgres_config, expected = shrinking

    DbShrinkerPostgres(postgres_config).run()

    actual = fetch_data(postgres_config)

    assert actual == expected
