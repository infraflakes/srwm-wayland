package main

import (
	"context"
	"fmt"
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

// BuildAll compiles the binary for all supported distros in parallel
// and returns a directory containing all of them.
func (m *SrwmWayland) BuildAll(ctx context.Context, source *dagger.Directory) *dagger.Directory {
	platforms := []string{"debian", "arch", "fedora"}

	// Start with an empty directory
	exports := dag.Directory()

	for _, p := range platforms {
		// We call the existing Build function for each platform
		// Dagger automatically handles the parallel execution of these branches
		binary := m.Build(ctx, source, p)

		// Add each binary to our export directory with a unique name
		exports = exports.WithFile(fmt.Sprintf("srwm-%s", p), binary)
	}

	return exports
}
