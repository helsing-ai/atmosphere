{
  description = "A demo of sqlite-web and multiple postgres services";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    systems.url = "github:nix-systems/default";
    process-compose-flake.url = "github:Platonic-Systems/process-compose-flake";
    services-flake.url = "github:juspay/services-flake";
    flake-parts.url = "github:hercules-ci/flake-parts";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = inputs @ { rust-overlay, nixpkgs, ...} :
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = import inputs.systems;

      imports = [
        inputs.process-compose-flake.flakeModule
      ];

      perSystem = { self', system, ... }: let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        dbName = "atmosphere";
      in {
        process-compose."default" = { config, ... }: {
          imports = [
            inputs.services-flake.processComposeModules.default
          ];

          services.postgres."pg1" = {
            enable = true;
            initialDatabases = [
              { name = dbName; }
            ];
          };

          settings.processes.test = let 
            pgconf = config.services.postgres.pg1;
          in {
            environment.DATABASE_URL = pgconf.connectionURI { inherit dbName; };

            command = pkgs.writeShellApplication {
              name = "cargo.test.postgres";
              runtimeInputs = with pkgs; [
                darwin.apple_sdk.frameworks.Security
                darwin.apple_sdk.frameworks.SystemConfiguration
                rust-bin.stable.latest.default
              ];
              text = "cargo test -F postgres";
            };
          };
        };
      };
    };
}
