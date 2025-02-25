{ pkgs ? import <nixpkgs> { } }:
let manifest = (pkgs.lib.importTOML ./Cargo.toml).package;
in
pkgs.rustPlatform.buildRustPackage rec {
  pname = manifest.name;
  version = manifest.version;

  cargoLock.lockFile = ./Cargo.lock;
  useFetchCargoVendor = true;

  nativeBuildInputs = with pkgs; [
    pkg-config
  ];
  buildInputs = with pkgs; [ systemd ];

  src = pkgs.lib.cleanSource ./.;

  meta = {
    description = "Turn your phone into a touchpad + graphics tablet with reaction of a fencer";
    homepage = "https://github.com/bpavuk/touche-driver";
    license = pkgs.lib.licenses.mit;
    maintainers = with pkgs.lib.maintainers; [ 
      # bpavuk
    ];
  };
}

