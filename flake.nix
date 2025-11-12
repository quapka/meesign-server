{
  description = "MeeSign Server";

  inputs = {
    nixpkgs.url      = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url  = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        nativeDependencies = with pkgs; [
            libpq
            pkg-config
            jdk17
            rust-bin.beta.latest.default
            protobuf
          ];
        buildDependencies = with pkgs; [
          openssl
        ];
      in
      with pkgs;
      {
        devShells.default = mkShell {
          buildInputs = buildDependencies ++ nativeDependencies;
        };

        packages.default = pkgs.rustPlatform.buildRustPackage rec {
          pname = "meesign-server";
          version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.version;
          src = ./.;
          cargoBuildFlags = "--package ${pname}";

          cargoLock = {
            lockFile = ./Cargo.lock;
            outputHashes = {
              "meesign-crypto-0.5.0" = "sha256-/ugNNNBnZTC58tTsXMbvwalMQh9sGgDkIFjUD+72T1A=";
            };
          };

          nativeBuildInputs = nativeDependencies;
          buildInputs = buildDependencies;
        };
      }
    );
}
