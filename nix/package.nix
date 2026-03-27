#with import <nixpkgs> {};
{ pkgs, lib, makeDesktopItem, rustPlatform, linux-wallpaperengine, ... }:

rustPlatform.buildRustPackage {
  pname = "wallpaper-engine";
  version = "0.1.1";

  src = lib.cleanSource ./..;
  cargoLock = {lockFile = ../Cargo.lock;};

  nativeBuildInputs = with pkgs; [ makeWrapper ];
  buildInputs = with pkgs; [ libxcb ];
  packages = with pkgs; [ linux-wallpaperengine ];

  postInstall = ''
    wrapProgram $out/bin/wallpaper-runner --prefix PATH : "${lib.makeBinPath [ linux-wallpaperengine ]}"

    wrapProgram $out/bin/wallpaper-gui --prefix PATH : "${lib.makeBinPath [ pkgs.libxkbcommon pkgs.libGL pkgs.wayland pkgs.wayland-protocols pkgs.wayland-scanner ]}"

    mkdir -p $out/share/applications
    cat > $out/share/applications/wallpaper-engine.desktop <<EOF
        [Desktop Entry]
        Type=Application
        Name=Wallpaper-engine
        Comment=Wallpaper manager
        Exec=$out/bin/wallpaper-gui %U
        Icon=wallpaper-gui
        Terminal=false
        EOF
  '';

  # This is literally ignored for no reason.
  #desktopItem = makeDesktopItem ({
  #  name = "Wallpaper manager";
  #  exec = "wallpaper-gui";
  #  icon = "wallpaper-gui";
  #  desktopName = "wallpaper-gui.desktop";
  #  comment = "Wallpaper manager";
  #  #categories = [ "Internet" ];
  #});

  meta = with lib; {
    description = "Wallpaper engine with runner and GUI";
    license = licenses.gpl3Only;
    platforms = platforms.linux;
    mainProgram = "wallpaper-gui";
  };
}