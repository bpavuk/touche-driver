{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
    crane = {
      url = "github:ipetkov/crane";
    };
  };
  outputs = { self, nixpkgs, flake-utils, rust-overlay, crane }:
    flake-utils.lib.eachSystem [ flake-utils.lib.system.x86_64-linux flake-utils.lib.system.aarch64-linux ] (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rustToolchain = pkgs.pkgsBuildHost.rust-bin.stable.latest.default;
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        src = craneLib.cleanCargoSource ./.;
        nativeBuildInputs = with pkgs; [ rustToolchain rust-analyzer pkg-config gh neovim lazygit ];
        buildInputs = with pkgs; [ systemd ];
        commonArgs = {
          inherit src buildInputs nativeBuildInputs;
        };
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        bin = craneLib.buildPackage (commonArgs // { inherit cargoArtifacts; });
      in
      with pkgs;
      {
        packages = {
          inherit bin;
          default = bin;
        };
        devShells.default = mkShell {
          inherit buildInputs nativeBuildInputs;
        };
      }
    );
}

