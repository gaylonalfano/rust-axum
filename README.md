## Starting the DB

```sh
# Start postgresql server docker image:
docker run --rm --name pg -p 5432:5432 \
  -e POSTGRES_PASSWORD=welcome \
  postgres:15

# (optional) To have a psql terminal on pg
# in another terminal run psql:
docker exec -it -u postgres pg psql

# (optional) For pg to print all sql statements,
# in psql command line started above.
ALTER DATABASE postgres SET log_statement = 'all';
```

### Postgres commands

```sh
\c app_db
\d [table]
app_db=# select * from "user";
```

## Dev (watch)

> NOTE: Install `cargo-watch` with: `cargo install cargo-watch`.

```sh
# Terminal 1 - To run the server
# NOTE: If we change ENV inside .cargo/config.rs,
# the server will auto-restart.
cargo watch -qcw src/ -w .cargo/ -x "run"

# Terminal 2 - To run the quick_dev
cargo watch -qcw examples/ -x "run --example quick_dev"
```

## Unit Test (watch)

```sh
cargo watch -qcx "test -- --nocapture"

# All tests in a package
cargo watch -qcx "test model::task::tests -- --nocapture"

# Specific test with filter
cargo watch -qcx "test model::task::tests::test_create_ok"

```
