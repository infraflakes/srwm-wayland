package main

import (
	"srwm-wayland/internal/dagger"
)

// base returns a container pre-configured with system dependencies for the target OS
func (m *SrwmWayland) base(os string) *dagger.Container {
	switch os {
	case "arch":
		return dag.Container().
			From("archlinux:latest").
			WithExec([]string{"pacman", "-Syu", "--noconfirm"}).
			WithExec([]string{"pacman", "-S", "--noconfirm",
				"pkgconf", "binutils", "gcc", "make", "git", "rust",
				"libdisplay-info", "libinput", "wayland", "libxkbcommon",
				"pixman", "libx11", "libxcursor", "libxrandr", "libxi",
				"libxcb", "mesa", "libglvnd", "seatd", "libdrm",
			})

	case "fedora":
		return dag.Container().
			From("fedora:latest").
			WithExec([]string{"dnf", "install", "-y",
				"pkgconf-pkg-config", "gcc", "gcc-c++", "git", "rust", "cargo",
				"libdisplay-info-devel", "libinput-devel", "wayland-devel",
				"libxkbcommon-devel", "pixman-devel", "libX11-devel",
				"libXcursor-devel", "libXrandr-devel", "libXi-devel",
				"libxcb-devel", "mesa-libGL-devel", "libseat-devel", "libdrm-devel",
				"mesa-libgbm-devel",
			})

	default:
		return dag.Container().
			From("rust:latest").
			WithEnvVariable("DEBIAN_FRONTEND", "noninteractive").
			WithExec([]string{"apt-get", "update"}).
			WithExec([]string{"apt-get", "install", "-y",
				"pkg-config", "git", "libseat-dev", "libdisplay-info-dev",
				"libinput-dev", "libudev-dev", "libgbm-dev", "libxkbcommon-dev",
				"libwayland-dev", "libdrm-dev", "libpixman-1-dev", "libx11-dev",
				"libxcursor-dev", "libxrandr-dev", "libxi-dev", "libxcb1-dev", "libgl-dev",
			})
	}
}
