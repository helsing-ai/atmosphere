{
  description = "Setup for testing Atmosphere";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    systems.url = "github:nix-systems/default";
    process-compose-flake.url = "github:Platonic-Systems/process-compose-flake";
    services-flake.url = "github:juspay/services-flake";
    flake-utils = {
      url = "github:numtide/flake-utils";
      inputs.systems.follows = "systems";
    };
    rust-overlay = {
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
      url = "github:oxalica/rust-overlay";
    };
  };

  outputs = inputs @ { rust-overlay, nixpkgs, ...} :
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = import inputs.systems;

      imports = [
        inputs.process-compose-flake.flakeModule
      ];

      perSystem = { self', system, lib, ... }: let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        dbName = "atmosphere";
        mySqlPort = 3310;
        ifDarwin = lib.optionals pkgs.stdenv.isDarwin;
      in {
        apps.lint.program = pkgs.writeShellApplication {
          name = "cargo.lint";
          runtimeInputs = with pkgs; [
            rust-bin.stable.latest.default
            typos
          ] ++ ifDarwin [
            darwin.apple_sdk.frameworks.SystemConfiguration
          ];
          text = ''
            set -eux

            cargo fmt --all
            cargo clippy
            typos
          '';
        };

        process-compose."default" = { config, ... }: {
          imports = [
            inputs.services-flake.processComposeModules.default
          ];

          settings.processes.test = let 
            pgconf = config.services.postgres.pg;
          in {
            command = pkgs.writeShellApplication {
              name = "cargo.test.postgres";
              runtimeInputs = with pkgs; [
                rust-bin.stable.latest.default
              ] ++ ifDarwin [
                darwin.apple_sdk.frameworks.SystemConfiguration
              ];
              text = ''
                set -eux

                mkdir -p data
                DATABASE_URL=${pgconf.connectionURI { inherit dbName; }} cargo test -F postgres
                DATABASE_URL=localhost:${toString mySqlPort} cargo test -F mysql
                DATABASE_URL=data/test.db cargo test -F sqlite
              '';
            };

            depends_on = {
              "pg".condition = "process_healthy";
              "mysql".condition = "process_healthy";
            };
          };

          services.postgres."pg" = {
            enable = true;
            initialDatabases = [
              { name = dbName; }
            ];
          };

          services.mysql."mysql" = {
            enable = true;
            initialDatabases = [
              { name = dbName; }
            ];
            settings.mysqld.port = mySqlPort;
          };

          # avoid both processes trying to create `data` directory at the same time
          settings.processes."mysql-configure".depends_on."pg-init".condition = "process_completed_successfully";
        };
        
      };
    };
}
