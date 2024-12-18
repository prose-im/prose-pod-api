# Running database migrations

First, set `DATABASE_URL` to a SQLite URL (e.g. `sqlite://database.sqlite?mode=rwc`).

## Generate a new migration file

```sh
# Doesn’t work because of our feature-oriented project structure.
# cargo run --bin migration -- generate MIGRATION_NAME
```

## Apply all pending migrations

```sh
cargo run --bin migration
```

```sh
cargo run --bin migration -- up
```

## Apply first 10 pending migrations

```sh
cargo run --bin migration -- up -n 10
```

## Rollback last applied migrations

```sh
cargo run --bin migration -- down
```

## Rollback last 10 applied migrations

```sh
cargo run --bin migration -- down -n 10
```

## Drop all tables from the database, then reapply all migrations

```sh
cargo run --bin migration -- fresh
```

## Rollback all applied migrations, then reapply all migrations

```sh
cargo run --bin migration -- refresh
```

## Rollback all applied migrations

```sh
cargo run --bin migration -- reset
```

## Check the status of all migrations

```sh
cargo run --bin migration -- status
```
