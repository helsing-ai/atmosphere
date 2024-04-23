# Contributing to `atmosphere`

We welcome contributions, big and small!

## Testing

Tests in `atmosphere` need a database to work with.
We provide a `docker-compose` service for this, you can use it in the following way:

```bash
$ docker-compose -f ./tests/postgres.yml up -d
$ export DATABASE_URL="postgres://atmosphere:atmosphere@localhost"
$ cargo test -F postgres
(... snip ...)
running 4 tests
test db::crud::read ... ok
test db::crud::create ... ok
test db::crud::delete ... ok
test db::crud::update ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
(... snip ...)
```
