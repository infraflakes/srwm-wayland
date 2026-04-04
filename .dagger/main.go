package main

import (
	"context"
	"srwm-wayland/internal/dagger"
)

type SrwmWayland struct{}

// Build compiles the binary for a specific OS.
// Usage: dagger call build --source=. --os=arch
func (m *SrwmWayland) Build(
	ctx context.Context,
	source *dagger.Directory,
	// +optional
	// +default="debian"
	os string,
) *dagger.File {

	return m.base(os).
		WithDirectory("/src", source.WithoutDirectory("target")).
		WithWorkdir("/src").
		WithExec([]string{"cargo", "build", "--release"}).
		File("target/release/srwm")
}
