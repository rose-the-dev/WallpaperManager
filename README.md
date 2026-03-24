# Premise
This is an educational project, to better learn Rust and UI design (I am terrible)

## Installation
### Nixos

Add the github as a flake input and add the module 'wallpaper-engine.homeManagerModules.wallpaper-engine' to imports / modules.
Enable the service with services.wallpaper-engine.enable = true;
This *should* add wallpaper-gui and wallpaper-runner to path.

**Currently broken:** wallpaper-gui doesn't run by default due to dependency issues (I honestly have no clue how to fix it)
but running it via "steam-run wallpaper-gui" does get it to run.
All I know is that the winit crate has issues with dynamically loading wayland client or something.

Any help is appreciated.

Example flake.nix:
```nix
{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    wallpaper-engine.url = "github:rose-the-dev/wallpaper-engine";
  };
  outputs = inputs@{ nixpkgs, home-manager, hyprland, wallpaper-engine, ... }:
  let
    system = "x86_64-linux";
    pkgs = nixpkgs.legacyPackages.${system};
  in {
    nixosConfigurations."<hostname>" = nixpkgs.lib.nixosSystem {
      specialArgs = {
        inherit system;
        inherit inputs;
      };
      modules = [
        ./configuration.nix
        home-manager.nixosModules.home-manager {
          home-manager = {
            useGlobalPkgs = true;
            useUserPackages = true;
            extraSpecialArgs = { inherit inputs; };
            users.rose = {
              imports = [
                ./home.nix
                wallpaper-engine.homeManagerModules.wallpaper-engine { services.wallpaper-engine.enable = true; }
              ];
            };
          };
        }
      ];
    };
  };
}
```

### Other distros
I do have plans to create a build and install script for other distros, but right now it's nixos only.


## Quirks
linux-wallpaper engine for some reason plays audio only when passed with the --silent flag, I built the wrapper app with the idea that --silent makes it... silent.
It does not.
