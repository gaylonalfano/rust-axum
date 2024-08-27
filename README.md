## TODOS

Following the progression here [ref](https://github.com/rust10x/rust-web-app?tab=readme-ov-file#rust10x-web-app-youtube-videos)

[x] Update to Axum 0.7
[x] Work through E02 with Sea-Query + SQLX + ModQL
[x] Update to multi-crate workspace

## LOG

2/6: Adding Sea Query + ModQL. [Quick summary of changes.](https://www.youtube.com/watch?v=-dMH9UiwKqg&list=PL7r-PXl6ZPcCIOFaL7nVHXZvBmHNhrh_Q)

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
# NOTE: -- --nocapture will print out the test results. Good for early dev.
cargo watch -qcx "test -- --nocapture"

# All tests in a package
cargo watch -qcx "test model::task::tests -- --nocapture"

# Specific test with filter
cargo watch -qcx "test model::task::tests::test_create_ok"

# Quick specific test while developing
cargo watch -qcx "test -q -p lib-auth test_multi_scheme -- --nocapture"

```

## Gen Key

```sh
cargo run -p gen-key
```
