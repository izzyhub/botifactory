# Things that will help with running
- [tracing-bunyan-formatter](https://github.com/LukeMathWalker/tracing-bunyan-formatter) (A way to make the stdout logs look better)
- [sqlx-cli](https://docs.rs/crate/sqlx-cli/0.5.7) Help get the database setup

## Installing them

```shell
cargo install tracing-bunyan-formatter sqlx-cli
```

# Preparing the project
## Setup your environment variables (direnv helps I find)
These are largely the same variable but the first is for sqlx-cli
and the second is for the actual app.
```shell
export DATABASE_URL="sqlite://$PWD/develop.db"
export BOTIFACTORY_APP__DATABASE__URL="file://$PWD/develop.db"
```
You can also do this
```shell
export DATABASE_URL="sqlite://$PWD/develop.db"
export BOTIFACTORY_APP__DATABASE__URL="$DATABASE_URL"
```

## Set up the database
```shell
sqlx database setup
```

### Run with formatted logs
cargo run | bunyan
