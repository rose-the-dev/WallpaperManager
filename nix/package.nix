#with import <nixpkgs> {};
{ pkgs, lib, makeDesktopItem, rustPlatform, linux-wallpaperengine, ... }:

rustPlatform.buildRustPackage {
  pname = "wallpaper-engine";
  version = "0.1.1";

  src = lib.cleanSource ./..;

  cargoLock = {lockFile = ../Cargo.lock;};
  #cargoHash = "sha256-KKi+r2D7bnJn8tVnjJx1x3jFsakijMQ8YKBFYBiB0RY=";

  nativeBuildInputs = with pkgs; [ makeWrapper ];
  buildInputs = with pkgs; [ libxcb ];
  packages = with pkgs; [ linux-wallpaperengine ];

  #postInstall = ''
  #  install -Dm755 target/release/wallpaper-runner $out/bin/wallpaper-runner
  #  install -Dm755 target/release/wallpaper-gui $out/bin/wallpaper-gui
  #'';

  postInstall = ''
    wrapProgram $out/bin/wallpaper-runner --prefix PATH : "${lib.makeBinPath [ linux-wallpaperengine ]}"

    wrapProgram $out/bin/wallpaper-gui --prefix PATH : "${lib.makeBinPath [ pkgs.wayland pkgs.wayland-protocols pkgs.wayland-scanner ]}"

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
    license = licenses.gpl3Only;
    platforms = platforms.linux;
    mainProgram = "wallpaper-gui";
  };
}