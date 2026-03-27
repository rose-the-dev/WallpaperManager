#with import <nixpkgs> {};
{ pkgs, lib, makeDesktopItem, rustPlatform, ... }:

rustPlatform.buildRustPackage {
  pname = "wallpaper-engine";
  version = "0.1.1";

  src = lib.cleanSource ./.;
  cargoLock = {lockFile = ./Cargo.lock;};

  #nativeBuildInputs = with pkgs; [ makeWrapper ];
  buildInputs = with pkgs; [ libxcb pkg-config libxkbcommon libxkbcommon.dev cairo cairo.dev glib ];

  #postInstall = ''
  #  wrapProgram $out/bin/wallpaper-runner --prefix PATH : "${lib.makeBinPath [ linux-wallpaperengine ]}"
  #'';

  meta = with lib; {
    description = "Wallpaper engine with runner and GUI";
    license = licenses.gpl3Only;
    platforms = platforms.linux;
    mainProgram = "wallpaper-engine";
  };
}