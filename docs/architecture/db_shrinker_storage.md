# Db Shrinker Storage

## Usage
`db-shrinker-postgres <connection-string>`

eg.

`db-shrinker-postgres 'postgresql://postgres:1234@localhost:5432/postgres'`

## Description
This binary merges all versions of documents stored in PostgreSQL into one, 'most recent' version.
It handles whole and partial updates to documents in mention.

## Testing
Currently only manual testing is supported.
You must have local postgres database provisioned for CDL document repository.

Setting up python env is done via `pip install -r tests/requirements.txt`.

Running `python data_loader.py` from tests directory should load sample data to your db. Just make sure that PSQL
connection string located in that file refers to your database instance.

After that you can run `db-shirnker-postgres` with same postgres connection string and compare data changed in your
database with `expected` entry in each test case.
