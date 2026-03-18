{
  description = "Wallpaper engine manager with systemd user service";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
  let
    pkgs = nixpkgs.legacyPackages."x86_64-linux";
    wallpaperPkg = pkgs.callPackage ./nix/package.nix { };
  in
  {
    packages."x86_64-linux".default = wallpaperPkg;
    overlays.default = final: prev: {
      wallpaper-manager = wallpaperPkg;
    };

    devShells.default = pkgs.mkShell rec {
      buildInputs = with pkgs; [
        pkg-config xcb
      ];
      packages = with pkgs; [  ];

      shellHook = ''
        echo "Bruh"
      '';

      PROJECT_NAME = "wallpaper-engine";
    };
    nixosModules.wallpaper-manager = import ./nix/module.nix;
    homeManagerModules.wallpaper-manager = import ./nix/module.nix;
  };
}