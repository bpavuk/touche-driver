{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    zig-overlay.url = "github:mitchellh/zig-overlay";
  };
  outputs = { self, nixpkgs, flake-utils, zig-overlay }:
    flake-utils.lib.eachSystem [ flake-utils.lib.system.x86_64-linux flake-utils.lib.system.aarch64-linux ] (system:
      let
        overlays = [ zig-overlay.overlays.default ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        nativeBuildInputs = with pkgs; [ 
          pkg-config 
          zigpkgs."0.16.0" 
          zls 
          linuxHeaders 
          glibc.dev 
        ];
        buildInputs = with pkgs; [ 
          libusb1.dev
          libevdev
        ];
      in
      with pkgs;
      {
        packages = {
          # inherit bin;
          # default = bin;
        };
        devShells.default = mkShell {
          buildInputs = buildInputs;
          nativeBuildInputs = nativeBuildInputs;

          LD_LIBRARY_PATH = lib.makeLibraryPath (nativeBuildInputs ++ buildInputs);
        };
      }
    );
}

