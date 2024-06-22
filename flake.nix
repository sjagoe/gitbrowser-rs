{
  description = "Dumb curses git browser for reading files from arbitraty git revisions ";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    systems.url = "github:nix-systems/default";
    utils = {
      url = "github:numtide/flake-utils";
      inputs.systems.follows = "systems";
    };
  };

  outputs = { self, nixpkgs, utils, ... }:
    (utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        inherit (pkgs) pkgsStatic;
      in
        {
          packages =
            {
              gitbrowser-rs =
                pkgs.rustPlatform.buildRustPackage {
                  pname = "gitbrowser-rs";
                  version = "0.1.0";
                  src = ./.;

                  cargoLock.lockFile = ./Cargo.lock;
                };
              gitbrowser-rs-static =
                pkgsStatic.rustPlatform.buildRustPackage {
                  pname = "gitbrowser-rs";
                  version = "0.1.0";
                  src = ./.;

                  cargoLock.lockFile = ./Cargo.lock;
                };
              default = self.packages.${system}.gitbrowser-rs;
            };

          apps = {
            default = self.apps."${system}".gitbrowser-rs;
            gitbrowser-rs = {
              type = "app";
              program = "${self.packages."${system}".default}/bin/gitbrowser-rs";
            };
          };

          devShells.default = pkgs.mkShell {
            inputsFrom = [ self.packages.${system}.gitbrowser-rs ];
            packages = with pkgs; [
              cargo
              rustc
              rust-analyzer
              rustfmt
              clippy
              rust.packages.stable.rustPlatform.rustLibSrc
            ];
          };
        })) // {
          overlays.default = final: prev: {
            gitbrowser-rs = self.packages.${prev.stdenv.hostPlatform.system}.gitbrowser-rs;
          };
        };
}
