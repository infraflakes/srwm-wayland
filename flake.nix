{
  description = "srwc — a trackpad-first infinite canvas Wayland compositor";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = {
    self,
    nixpkgs,
  }: let
    system = "x86_64-linux";
    pkgs = nixpkgs.legacyPackages.${system};

    nativeBuildInputs = with pkgs; [
      pkg-config
      rustPlatform.bindgenHook
    ];

    buildInputs = with pkgs; [
      wayland
      wayland-protocols
      seatd
      libdisplay-info
      libinput
      libgbm
      libxkbcommon
      libdrm
      systemd
      libglvnd
      libx11
      libxcursor
      libxrandr
      libxi
      libxcb
      pixman
      dbus
      pipewire
    ];

    runtimeLibs = with pkgs; [
      wayland
      seatd
      libdisplay-info
      libinput
      libgbm
      libxkbcommon
      libdrm
      systemd
      libglvnd
      libx11
      libxcursor
      libxrandr
      libxi
      libxcb
      pixman
      dbus
      pipewire
    ];
  in {
    packages.${system}.default = pkgs.rustPlatform.buildRustPackage rec {
      pname = "srwc";
      version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.version;

      src = pkgs.lib.cleanSourceWith {
        src = ./.;
        filter = path: type: let
          baseName = builtins.baseNameOf path;
        in
          baseName != "target" && baseName != ".git" && baseName != ".direnv";
      };

      cargoLock = {
        lockFile = ./Cargo.lock;
        allowBuiltinFetchGit = true;
      };

      inherit nativeBuildInputs buildInputs;

      env = {
        RUSTFLAGS = toString (
          map (arg: "-C link-arg=" + arg) [
            "-Wl,--push-state,--no-as-needed"
            "-lEGL"
            "-lwayland-client"
            "-Wl,--pop-state"
          ]
        );
      };

      postFixup = ''
        patchelf --add-rpath "${pkgs.lib.makeLibraryPath runtimeLibs}" $out/bin/srwc
      '';

      postInstall = ''
        install -Dm644 resources/srwc.desktop $out/share/wayland-sessions/srwc.desktop
        install -Dm644 resources/srwc-portals.conf $out/share/xdg-desktop-portal/srwc-portals.conf
        install -Dm644 resources/config.example.toml $out/etc/srwc/config.toml
        for f in resources/extras/wallpapers/*.glsl; do
          install -Dm644 "$f" "$out/share/srwc/wallpapers/$(basename "$f")"
        done
      '';

      passthru.providedSessions = ["srwc"];

      meta = with pkgs.lib; {
        description = "A trackpad-first infinite canvas Wayland compositor";
        license = licenses.gpl3Plus;
        platforms = ["x86_64-linux"];
        mainProgram = "srwc";
      };
    };

    devShells.${system}.default = pkgs.mkShell {
      inherit nativeBuildInputs buildInputs;

      LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath runtimeLibs;
    };
  };
}
