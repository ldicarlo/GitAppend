{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
    crane.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, crane, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config.allowUnfree = true;
        };
        lib = import lib { };
        craneLib = crane.lib.${system};
        code = pkgs.callPackage ./. { inherit nixpkgs system; };
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

        packages = rec {
          default = git-append;
          # https://crane.dev/getting-started.html
          git-append = craneLib.buildPackage rec {
            name = "git-append";
            src = craneLib.cleanCargoSource ./.;
            rust-dependencies = craneLib.buildDepsOnly {
              inherit src;
              buildInputs = with pkgs; [ pkg-config openssl ];
            };

            rust-package-binary = craneLib.buildPackage {
              inherit src;
              cargoArtifacts = rust-dependencies;
              buildInputs = with pkgs; [ pkg-config openssl ];
              doCheck = true;
              checkPhase = ''
                runHook preCheck
                cp -r tests $out/
                cargo test
              '';
            };
          };
        };
      });
}
