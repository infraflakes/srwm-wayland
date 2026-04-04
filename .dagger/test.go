package main

import (
	"context"
	"fmt"
	"golang.org/x/sync/errgroup"
	"srwm-wayland/internal/dagger"
)

// TestAll runs cargo test across all supported distributions
func (m *SrwmWayland) TestAll(ctx context.Context, source *dagger.Directory) error {
	platforms := []string{"debian", "arch", "fedora"}
	g, ctx := errgroup.WithContext(ctx)

	for _, p := range platforms {
		platform := p // Shadow for closure
		g.Go(func() error {
			fmt.Printf("Testing on %s...\n", platform)
			_, err := m.base(platform).
				WithDirectory("/src", source.WithoutDirectory("target")).
				WithWorkdir("/src").
				WithExec([]string{"cargo", "test"}).
				Sync(ctx)
			return err
		})
	}

	return g.Wait()
}
