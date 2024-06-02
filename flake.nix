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
          };
          lib = pkgs.lib;
          craneLib = crane.mkLib nixpkgs.legacyPackages.${system};

          commonArgs = {
            src = lib.cleanSourceWith {
              src = ./.;
              filter = path: type:
                (lib.hasInfix "tests/" path) ||
                (craneLib.filterCargoSources path type)
              ;
            };
            buildInputs = with pkgs; [ pkg-config openssl ];
          };
          cargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
            pname = "git-append";
          });

          git-append = craneLib.buildPackage (commonArgs // {
            pname = "git-append";
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

          packages =
            rec  {
              default = git-append;
            };


          nixosModules.default = { config, lib, pkgs, ... }:
            with lib;
            let
              cfg = config.services.git-append;
              git-append = self.packages.${system}.git-append;
            in
            rec
            {
              options.services.git-append = {
                enable = mkEnableOption "Enable git-append service";
                configFile = mkOption {
                  type = types.path;
                  description = "The location of the config file. Check the doc for the details.";
                };
              };
              config = mkIf cfg.enable
                {
                  systemd.services.git-append = {
                    description = "git-append runner";
                    wantedBy = [ "multi-user.target" ];
                    environment = { };
                    serviceConfig = {
                      ExecStart = "${git-append}/bin/git-append run --config-path=${configFile}";
                      Restart = "on-failure";
                      RestartSec = "10s";
                    };
                  };
                  systemd.timers.git-append = {
                    enable = true;
                    unitConfig = {
                      description = "Git Append timer";
                      after = [ "network.target" ];
                    };
                    timerConfig = {
                      OnBootSec = "5 min";
                      OnUnitInactiveSec = "10 sec";
                    };
                  };

                };
            };

          nixosModules.homeManagerModule = { config, lib, pkgs, ... }:
            with lib;
            let
              cfg = config.services.git-append;
              git-append-package = self.packages.${system}.git-append;
            in
            {
              options.services.git-append = {
                enable = mkEnableOption "Enable git-append service";
                configFile = mkOption {
                  type = types.path;
                  description = "The location of the config file. Check the doc for the details.";
                };
              };
              config = mkIf cfg.enable
                {
                  systemd.user.services.git-append = {
                    Unit = {
                      Description = "git-append runner";
                    };
                    Service = {
                      ExecStart = "${git-append}/bin/git-append run --config-path=${cfg.configFile}";
                      Restart = "on-failure";
                      RestartSec = "10s";
                      Type = "oneshot";
                    };
                    Install = {
                      WantedBy = [ "timers.target" ];
                    };
                  };
                  systemd.user.timers.git-append = {
                    Unit = {
                      Description = "Git Append timer";
                    };
                    Timer = {
                      After = [ "network.target" ];
                      OnBootSec = "5 min";
                      OnUnitInactiveSec = "10 sec";
                    };
                    Install = {
                      WantedBy = [ "timers.target" ];
                    };
                  };

                };
            };
        });
}
