with import <nixpkgs> {};
stdenv.mkDerivation {
    name = "dev-environment";
    buildInputs = [ pkg-config libxkbcommon libxkbcommon.dev cairo cairo.dev glib glib.dev ffmpeg ];
}
