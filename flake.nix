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
          librsvg
          gsettings-desktop-schemas
          adwaita-icon-theme
          hicolor-icon-theme
          wayland
        ];

        # CLI utilities called by akquick services (audio control, etc.)
        runtimeTools = with pkgs; [
          wireplumber
          pulseaudio
        ];

        # Dev utilities (caching and fast linking)
        devTools = with pkgs; [
          sccache
          mold
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          inherit nativeBuildInputs;
          buildInputs = buildInputs ++ runtimeTools ++ devTools;

          # Provide standard library source for rust-analyzer
          RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";

          # sccache configuration: shared cache with automatic LRU Garbage Collection
          RUSTC_WRAPPER = "${pkgs.sccache}/bin/sccache";
          SCCACHE_DIR = "/var/home/phamnv/.cache/sccache";
          SCCACHE_CACHE_SIZE = "15G";

          shellHook = ''
            export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath (buildInputs ++ runtimeTools)}:$LD_LIBRARY_PATH"
            export PKG_CONFIG_PATH="${pkgs.lib.makeSearchPathOutput "dev" "lib/pkgconfig" buildInputs}:$PKG_CONFIG_PATH"
            export GDK_PIXBUF_MODULE_FILE="${pkgs.librsvg}/lib/gdk-pixbuf-2.0/2.10.0/loaders.cache"
            export XDG_DATA_DIRS="${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}:${pkgs.gtk4}/share/gsettings-schemas/${pkgs.gtk4.name}:${pkgs.adwaita-icon-theme}/share:${pkgs.hicolor-icon-theme}/share:$XDG_DATA_DIRS"
            export RUSTFLAGS="-C link-arg=-fuse-ld=mold $RUSTFLAGS"
          '';
        };

        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "akquick";
          version = "0.1.0";
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
          nativeBuildInputs = nativeBuildInputs ++ [ pkgs.mold ];
          inherit buildInputs;
          RUSTFLAGS = "-C link-arg=-fuse-ld=mold";
          doCheck = false;

          preFixup = ''
            gappsWrapperArgs+=(
              --prefix PATH : "${pkgs.lib.makeBinPath runtimeTools}"
            )
          '';
        };
      }
    );
}
