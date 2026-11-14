{
  description = "botifactory — release distribution service";

  # nixos-unstable, not a release channel: sqlx is pinned to 0.9, which needs a
  # recent rustc, and the matching sqlx-cli is only in unstable.
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { self, nixpkgs }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
      ];
      forAllSystems =
        f: nixpkgs.lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});
    in
    {
      packages = forAllSystems (
        pkgs:
        let
          botifactory = pkgs.callPackage ./nix/package.nix { };
        in
        {
          inherit botifactory;
          default = botifactory;
        }
        # dockerTools images are Linux-only; on darwin the attribute is absent.
        // pkgs.lib.optionalAttrs pkgs.stdenv.hostPlatform.isLinux {
          image = pkgs.callPackage ./nix/image.nix { inherit botifactory; };
        }
      );

      overlays.default = final: _prev: {
        botifactory = final.callPackage ./nix/package.nix { };
      };

      # `nix flake check`: the build (offline sqlx data + unit tests) and the
      # lint gates, each as its own derivation so failures are reported apart.
      checks = forAllSystems (
        pkgs:
        let
          botifactory = self.packages.${pkgs.stdenv.hostPlatform.system}.botifactory;
        in
        {
          inherit botifactory;

          clippy = botifactory.overrideAttrs (old: {
            pname = "botifactory-clippy";
            nativeBuildInputs = old.nativeBuildInputs ++ [ pkgs.clippy ];
            buildPhase = ''
              runHook preBuild
              cargo clippy --offline --all-targets -- -D warnings
              runHook postBuild
            '';
            doCheck = false;
            installPhase = "touch $out";
            postInstall = "";
          });

          fmt =
            pkgs.runCommand "botifactory-fmt"
              {
                nativeBuildInputs = [
                  pkgs.cargo
                  pkgs.rustfmt
                ];
              }
              ''
                cd ${./.}
                cargo fmt --check
                touch $out
              '';
        }
      );

      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShell {
          packages = [
            pkgs.cargo
            pkgs.rustc
            pkgs.clippy
            pkgs.rustfmt
            pkgs.rust-analyzer
            # `sqlx database setup` / `cargo sqlx prepare`. Tracks nixpkgs'
            # sqlx-cli, which must match the sqlx major in Cargo.toml.
            pkgs.sqlx-cli
            pkgs.sqlite
          ];

          env.RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";

          shellHook = ''
            export DATABASE_URL="''${DATABASE_URL:-sqlite://$PWD/develop.db}"
            export BOTIFACTORY_APP__DATABASE__URL="''${BOTIFACTORY_APP__DATABASE__URL:-$DATABASE_URL}"
          '';
        };
      });
    };
}
