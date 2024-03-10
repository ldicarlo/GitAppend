{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
    crane.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, crane, flake-utils }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          pkgs = import nixpkgs {
            inherit system;
            config.allowUnfree = true;
          };
          lib = import lib { };
          craneLib = crane.lib.${system};

          commonArgs = {
            src = lib.cleanSourceWith {
              src = ./.;
              filter = path: type:
                (lib.hasInfix "tests/" path) ||
                (craneLib.filterCargoSources path type)
              ;
            };
            strictDeps = true;
            buildInputs = with pkgs; [ pkg-config openssl ];
          };
          cargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
            pname = "mycrate-deps";
          });

          git-append = craneLib.buildPackage (commonArgs // {
            inherit cargoArtifacts;
          });
        in
        rec {
          devShells.default = pkgs.mkShell {
            buildInputs = with pkgs; [
              rustc
              rustfmt
              cargo
              pkg-config
              openssl
            ];
          };

          #     checks = { inherit git-append; };
          packages =
            {
              default = git-append;
            };


          nixosModules.default = { config, lib, pkgs, ... }:
            with lib;
            let
              cfg = config.services.git-append;
              git-append = self.packages.${system}.git-append;
            in
            {
              options.services.git-append = {
                enable = mkEnableOption "Enable git-append service";
                envFile = mkOption { type = types.str; };
              };
              config = mkIf cfg.enable
                {
                  systemd.services.git-append = {
                    description = "git-append runner";
                    wantedBy = [ "multi-user.target" ];
                    environment = { };
                    serviceConfig = {
                      ExecStart = "${git-append}/bin/git-append";
                      Restart = "on-failure";
                      RestartSec = "10s";
                    };
                  };

                };
            };
        });
}
