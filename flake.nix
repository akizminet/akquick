{
  description = "akquick - Modern Quick Settings & Control Center for Wayland";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        nativeBuildInputs = with pkgs; [
          pkg-config
          rustc
          cargo
          rustfmt
          clippy
          rust-analyzer
          wrapGAppsHook4
        ];
        buildInputs = with pkgs; [
          glib
          gtk4
          libadwaita
          gtk4-layer-shell
          pango
          cairo
          gdk-pixbuf
          wayland
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          inherit nativeBuildInputs buildInputs;

          shellHook = ''
            export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath buildInputs}:$LD_LIBRARY_PATH"
            export PKG_CONFIG_PATH="${pkgs.lib.makeSearchPathOutput "dev" "lib/pkgconfig" buildInputs}:$PKG_CONFIG_PATH"
          '';
        };

        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "akquick";
          version = "0.1.0";
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
          inherit nativeBuildInputs buildInputs;
        };
      }
    );
}
