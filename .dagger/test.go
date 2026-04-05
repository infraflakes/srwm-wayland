package main

import (
	"context"
	"srwm-wayland/internal/dagger"
)

// Test runs cargo test using the standard Rust environment.
func (m *SrwmWayland) Test(ctx context.Context, source *dagger.Directory) *dagger.Container {
	return dag.Container().
		From("rust:latest").
		WithEnvVariable("DEBIAN_FRONTEND", "noninteractive").
		WithExec([]string{"apt-get", "update"}).
		WithExec([]string{"apt-get", "install", "-y",
			"pkg-config", "git", "libseat-dev", "libdisplay-info-dev",
			"libinput-dev", "libudev-dev", "libgbm-dev", "libxkbcommon-dev",
			"libwayland-dev", "libdrm-dev", "libpixman-1-dev", "libx11-dev",
			"libxcursor-dev", "libxrandr-dev", "libxi-dev", "libxcb1-dev", "libgl-dev",
			"libpipewire-0.3-dev", "libclang-dev",
		}).
		WithExec([]string{"rustup", "component", "add", "clippy", "rustfmt"}).
		WithDirectory("/src", source.WithoutDirectory("target")).
		WithWorkdir("/src").
		WithExec([]string{"cargo", "test"}).
		WithExec([]string{"cargo", "clippy"}).
		WithExec([]string{"cargo", "fmt", "--check"})
}
