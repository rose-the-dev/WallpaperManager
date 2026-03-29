{
  description = "Wallpaper engine manager with systemd user service";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
  };

  outputs = { self, nixpkgs }:
  let
    pkgs = nixpkgs.legacyPackages."x86_64-linux";
    wallpaper-engine = pkgs.callPackage ./nix/package.nix { };
    #wallpaper-engine = pkgs.callPackage ./wallpaper-engine/default.nix { };
    #wallpaper-manager = pkgs.callPackage ./wallpaper-manager/default.nix { };
    #wallpaper-ctl = pkgs.callPackage ./wallpaper-ctl/default.nix { };
  in
  {
    packages."x86_64-linux".default = wallpaper-engine;
    #packages."x86_64-linux".manager = wallpaper-manager;
    #packages."x86_64-linux".ctl = wallpaper-ctl;
    overlays.default = final: prev: {
      wallpaper-engine = wallpaper-engine;
      #wallpaper-manager = wallpaper-manager;
      #wallpaper-ctl = wallpaper-ctl;
    };
    #nixosModules.wallpaper-engine = import ./nix/module.nix;
    homeManagerModules.wallpaper-engine = import ./nix/hm-module.nix;
  };
}