#with import <nixpkgs> {};
{ pkgs, lib, makeDesktopItem, rustPlatform, ... }:

rustPlatform.buildRustPackage {
  pname = "wallpaper-manager";
  version = "0.1.0";

  src = lib.cleanSource ./..;

  cargoLock = {lockFile = ../Cargo.lock;};
  #cargoHash = "sha256-KKi+r2D7bnJn8tVnjJx1x3jFsakijMQ8YKBFYBiB0RY=";

  buildInputs = with pkgs; [ pkg-config linux-wallpaperengine libxcb wayland wayland-protocols wayland-scanner ];

  #postInstall = ''
  #  install -Dm755 target/release/wallpaper-runner $out/bin/wallpaper-runner
  #  install -Dm755 target/release/wallpaper-gui $out/bin/wallpaper-gui
  #'';

  desktopItems = [
      (makeDesktopItem {
        name = "Wallpaper manager";
        exec = "wallpaper-gui";
        icon = "wallpaper-gui";
        desktopName = "wallpaper-gui.desktop";
        comment = "Wallpaper manager";
        #categories = [ "Internet" ];
      })
    ];

  meta = with lib; {
    description = "Wallpaper engine with runner and GUI";
    license = licenses.mit;
    platforms = platforms.linux;
    mainProgram = "wallpaper-gui";
  };
}