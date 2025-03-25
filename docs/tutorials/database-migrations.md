# Running database migrations

First, set `DATABASE_URL` to a SQLite URL (e.g. `sqlite://database.sqlite?mode=rwc`).

## Generate a new migration file

```sh
# Doesn’t work because of our feature-oriented project structure.
# cargo run --bin migrator -- generate MIGRATION_NAME
```

## Apply all pending migrations

```sh
cargo run --bin migrator
```

```sh
cargo run --bin migrator -- up
```

## Apply first 10 pending migrations

```sh
cargo run --bin migrator -- up -n 10
```

## Rollback last applied migrations

```sh
cargo run --bin migrator -- down
```

## Rollback last 10 applied migrations

```sh
cargo run --bin migrator -- down -n 10
```

## Drop all tables from the database, then reapply all migrations

```sh
cargo run --bin migrator -- fresh
```

## Rollback all applied migrations, then reapply all migrations

```sh
cargo run --bin migrator -- refresh
```

## Rollback all applied migrations

```sh
cargo run --bin migrator -- reset
```

## Check the status of all migrations

```sh
cargo run --bin migrator -- status
```
