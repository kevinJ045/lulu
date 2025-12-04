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
            pname = "lulu";
            version = "0.1.0";
            src = ./.;
            cargoLock = {
              lockFile = ./Cargo.lock;
            };

            nativeBuildInputs = with pkgs; [
              pkg-config
            ];

            buildInputs = with pkgs; [
              luajit
            ];

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
              luajit
            ];

            shellHook = ''
            export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath [pkgs.luajit]}:$LD_LIBRARY_PATH
            '';
          };
        }
      );
    };
}
