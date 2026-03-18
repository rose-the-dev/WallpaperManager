{ config, lib, pkgs, ... }:

with lib;

let
  cfg = config.services.wallpaper-engine;
  wallpaperPkg = pkgs.callPackage ./package.nix { };
in
{
  options.services.wallpaper-engine = {
    enable = mkEnableOption "Enable the wallpaper-runner user service";

    package = mkOption {
      type = types.package;
      default = wallpaperPkg;
      description = "The wallpaper-engine package to use.";
    };
  };

  config = mkIf cfg.enable {
    systemd.user.services.wallpaper-runner = {
      Unit.Description = "Wallpaper Runner Service";
      Unit.After = [ "graphical-session.target" ];

      Service.Type = "simple";
      Service.ExecStart = "${cfg.package}/bin/wallpaper-runner";
      Service.Restart = "always";
      Service.RestartSec = 5;

      Install.WantedBy = [ "default.target" ];
    };
    #environment.systemPackages = [ cfg.package ];
    home.packages = [ cfg.package ];
  };
}