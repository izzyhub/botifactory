# Things that will help with running
- [tracing-bunyan-formatter](https://github.com/LukeMathWalker/tracing-bunyan-formatter) (A way to make the stdout logs look better)
- [sqlx-cli](https://docs.rs/crate/sqlx-cli/0.5.7) Help get the database setup

## Installing them

```shell
cargo install tracing-bunyan-formatter sqlx-cli
```

# Preparing the project
## Set up the database
```shell
sqlx database setup
```

### Run with formatted logs
cargo run | bunyan
