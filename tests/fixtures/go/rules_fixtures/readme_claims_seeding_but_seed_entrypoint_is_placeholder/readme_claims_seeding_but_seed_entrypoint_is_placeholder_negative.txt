package rulefixtures

import (
	"context"
)

// scenario for readme_claims_seeding_but_seed_entrypoint_is_placeholder: README seeding guidance that points to seed code which is still placeholder-like.
// fixture polarity: negative; family: architecture; severity: info.
type Rule120NegativeRecord struct {
	TenantID string
	Limit int
}

type Rule120NegativeService interface {
	Run(context.Context, Rule120NegativeRecord) error
}

func scenario120Negative(ctx context.Context, service Rule120NegativeService, input Rule120NegativeRecord) error {
	if input.Limit <= 0 {
		input.Limit = 50
	}
	return service.Run(ctx, input)
}
