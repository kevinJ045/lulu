{
  description = "Lulu Flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.05";
  };

  outputs =
    { self, nixpkgs }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      shaTable = {
        x86_64-linux = "sha256-9M67FZ4TzjoiGL73B8Jtwn38lW521yCLIqyvGzYCc50=";
        aarch64-linux = "sha256-J4E32qZNyqmJyFKBuU+6doRYL3ZSaEMSBlML+hSkj+o=";
        x86_64-darwin = "sha256-UnulsDS1LlrVR2+cz+4zgWxKqbkB5ch3T9UofGCZduQ=";
        aarch64-darwin = "sha256-mU7N/1vXzCP+mwjzLTsDkT+8YOJifwNju3Rv9Cq5Loo=";
      };

      forEachSystem =
        f:
        nixpkgs.lib.genAttrs systems (
          system:
          let
            pkgs = import nixpkgs { inherit system; };
            clang = pkgs.llvmPackages_16.clang;
            rustPlatform = pkgs.rustPlatform;
            version = "137.2.0";
          in
          f {
            inherit
              pkgs
              system
              clang
              rustPlatform
              ;
          }
        );
    in
    {
      packages = forEachSystem (
        {
          pkgs,
          system,
          rustPlatform,
          clang,
        }:
        {
          default = rustPlatform.buildRustPackage {
            pname = "rew_runtime";
            version = "0.1.0";
            src = ./.;
            cargoLock = {
              lockFile = ./Cargo.lock;
            };

            nativeBuildInputs = with pkgs; [
              pkg-config
            ];

            buildInputs = [ ];

          };
        }
      );

      devShells = forEachSystem (
        { pkgs, system, ... }:
        {
          default = pkgs.mkShell {
            nativeBuildInputs = with pkgs; [
              cargo
              clippy
              pkg-config
              rust-analyzer
              rustc
              rustfmt
            ];
          };
        }
      );
    };
}
