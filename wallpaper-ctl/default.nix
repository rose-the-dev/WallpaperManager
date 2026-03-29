#with import <nixpkgs> {};
{ pkgs, lib, rustPlatform, ... }:

rustPlatform.buildRustPackage {
  pname = "wallpaper-ctl";
  version = "0.1.1";

  src = lib.cleanSource ./.;
  cargoLock = {lockFile = ../Cargo.lock;};

  nativeBuildInputs = with pkgs; [  ];

  meta = with lib; {
    description = "Wallpaper engine with runner and GUI";
    license = licenses.gpl3Only;
    platforms = platforms.linux;
    mainProgram = "wallpaper-ctl";
  };
}