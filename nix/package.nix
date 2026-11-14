# The botifactory binary, plus its configuration/ directory under share/.
#
#  - `SQLX_OFFLINE=true` is mandatory: the `sqlx::query!` macros check SQL
#    against a live database at compile time, and the Nix sandbox has none, so
#    the build reads the committed `.sqlx/` query data instead. A stale `.sqlx/`
#    fails here — rerun `cargo sqlx prepare` after changing a query.
#  - The unit tests need no database, so they run as the check phase.
#  - The botifactory-* dependencies are git dependencies; allowBuiltinFetchGit
#    fetches them by the revisions pinned in Cargo.lock, so there are no
#    outputHashes to keep in sync when they are bumped.
{
  lib,
  rustPlatform,
  pkg-config,
  aws-lc,
  sqlite,
}:

rustPlatform.buildRustPackage {
  pname = "botifactory";
  version = "0.1.0";

  # Only what the compiler reads, so editing docs or CI does not rebuild.
  # migrations/ is embedded by `sqlx::migrate!()`; configuration/ is read at
  # run time from the working directory and is installed alongside the binary.
  src = lib.fileset.toSource {
    root = ../.;
    fileset = lib.fileset.unions [
      ../Cargo.toml
      ../Cargo.lock
      ../build.rs
      ../src
      ../migrations
      ../.sqlx
      ../configuration
    ];
  };

  cargoLock = {
    lockFile = ../Cargo.lock;
    allowBuiltinFetchGit = true;
  };

  env.SQLX_OFFLINE = "true";

  # Both link system libraries: sqlx's and rusqlite's libsqlite3-sys, and
  # aws-lc-sys (rustls, via botifactory-common's reqwest).
  nativeBuildInputs = [ pkg-config ];
  buildInputs = [
    sqlite
    aws-lc
  ];

  postInstall = ''
    mkdir -p $out/share/botifactory
    cp -r configuration $out/share/botifactory/
  '';

  meta = {
    description = "Release distribution service for firmware binaries";
    mainProgram = "botifactory";
    platforms = lib.platforms.unix;
  };
}
