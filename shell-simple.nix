{pkgs ? import <nixpkgs> {}}: let
  overrides = builtins.fromTOML (builtins.readFile ./rust-toolchain.toml);
in
  pkgs.mkShell rec {
    nativeBuildInputs = with pkgs; [
      pkg-config
    ];
    buildInputs = with pkgs; [
      rustc
      cargo
      udev
      alsa-lib
      vulkan-loader
      libxkbcommon
      wayland
      xorg.libX11
      xorg.libXcursor
      xorg.libXi
      xorg.libXrandr
    ];
    shellHook = ''
      export PATH=$PATH:''${CARGO_HOME:-~/.cargo}/bin
      echo ""
      echo "Welcome to your Rust-Bevy development environment!" | ${pkgs.lolcat}/bin/lolcat
      echo "Start your IDE with: nohup jetbrains-toolbox &"
      echo ""
    '';
  }
