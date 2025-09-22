{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{ fenix, flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" ];
      perSystem =
        { pkgs, system, ... }:

        let
          toolchain = fenix.packages."${system}".complete.toolchain;
        in
        {
          _module.args.pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [
              (
                _: super:
                let
                  pkgs = fenix.inputs.nixpkgs.legacyPackages.${super.system};
                in
                fenix.overlays.default pkgs pkgs
              )
            ];
          };

          devShells.default = pkgs.mkShell {
            nativeBuildInputs = [ toolchain ];

            packages = [
              pkgs.cargo-shuttle
              pkgs.rust-analyzer-nightly
              pkgs.tailwindcss_4
              pkgs.esbuild
              pkgs.sqlx-cli
              pkgs.sqlite
            ];

            DATABASE_URL = "sqlite:dev.db";
          };
        };
    };

  nixConfig = {
    extra-substituters = [
      "https://nix-community.cachix.org"
    ];
    extra-trusted-public-keys = [
      "nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs="
    ];
  };
}
