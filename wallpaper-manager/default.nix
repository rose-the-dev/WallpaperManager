#with import <nixpkgs> {};
{ pkgs, lib, makeDesktopItem, rustPlatform, ... }:

rustPlatform.buildRustPackage {
  pname = "wallpaper-manager";
  version = "0.1.1";

  src = lib.cleanSource ./.;
  cargoLock = {lockFile = ./Cargo.lock;};

  nativeBuildInputs = with pkgs; [ makeWrapper ];
  buildInputs = with pkgs; [ libxcb ];

  postInstall = ''
    wrapProgram $out/bin/wallpaper-manager --prefix PATH : "${lib.makeBinPath [ pkgs.libxkbcommon pkgs.libGL pkgs.wayland pkgs.wayland-protocols pkgs.wayland-scanner ]}"

    mkdir -p $out/share/applications
    cat > $out/share/applications/wallpaper-manager.desktop <<EOF
        [Desktop Entry]
        Type=Application
        Name=Wallpaper-engine
        Comment=Wallpaper manager
        Exec=$out/bin/wallpaper-manager %U
        Icon=wallpaper-manager
        Terminal=false
        EOF
  '';

  # This is literally ignored for no reason.
  #desktopItem = makeDesktopItem ({
  #  name = "Wallpaper manager";
  #  exec = "wallpaper-manager";
  #  icon = "wallpaper-manager";
  #  desktopName = "wallpaper-manager.desktop";
  #  comment = "Wallpaper manager";
  #});

  meta = with lib; {
    description = "Wallpaper engine with runner and GUI";
    license = licenses.gpl3Only;
    platforms = platforms.linux;
    mainProgram = "wallpaper-manager";
  };
}