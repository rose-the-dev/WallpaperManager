{ stdenv, linux-wallpaperengine }:
stdenv.mkDerivation {
  pname = "wallpaper-gui";
  version = "0.0.1";

  src = "./";

  buildInputs = with pkgs; [ linux-wallpaperengine ];
}