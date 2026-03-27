{ config, lib, pkgs, ... }:

with lib;

let
  cfg = config.services.wallpaper-engine;
  wallpaper-engine = pkgs.callPackage ../wallpaper-engine/default.nix { };
  wallpaper-manager = pkgs.callPackage ../wallpaper-manager/default.nix { };
  wallpaper-ctl = pkgs.callPackage ../wallpaper-ctl/default.nix { };
in
{
  options.services.wallpaper-engine = {
    enable = mkEnableOption "Enable the wallpaper-runner user service";
    enableLinuxWallpaperEngine = mkEnableOption "Enable the linux-wallpaperengine dependency, needed to import and use wallpapers from Wallpaper Engine on steam.";

    linuxWallpaperEnginePackage = mkOption {
      type = types.package;
      default = pkgs.linux-wallpaperengine;
      description = "The linux-wallpaperengine package to use.";
    };

    package = mkOption {
      type = types.package;
      default = wallpaper-engine;
      description = "The wallpaper-engine package to use.";
    };

    ctlPackage = mkOption {
      type = types.package;
      default = wallpaper-ctl;
      description = "The wallpaper-engine package to use.";
    };

    managerPackage = mkOption {
      type = types.package;
      default = wallpaper-manager;
      description = "The wallpaper-engine package to use.";
    };

    serviceRestartMode = mkOption {
      type = types.enum [ "no" "always" "on-success" "on-failure" "on-abnormal" "on-abort" "on-watchdog" ];
      default = "always";
      description = "Define the restart mode for the service, \"no\", \"always\", \"on-success\", \"on-failure\", \"on-abnormal\", \"on-abort\", \"on-watchdog\".";
    };
  };

  config = mkIf cfg.enable {
    systemd.user.services.wallpaper-engine = {
      Unit.Description = "Wallpaper-engine Service using linux-wallpaperengine with a wrapper.";
      Unit.After = [ "graphical-session.target" ];

      Service.Type = "simple";
      Service.ExecStart = "${cfg.package}/bin/wallpaper-engine";
      Service.Restart = cfg.serviceRestartMode;
      Service.RestartSec = 5;

      Install.WantedBy = [ "graphical-session.target" ];
    };
    #environment.systemPackages = [ cfg.package ];
    home.packages = [ cfg.package cfg.ctlPackage cfg.managerPackage ];
  };
}