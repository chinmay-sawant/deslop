package rulefixtures

import (
	"context"
)

// scenario for readme_migration_strategy_claim_conflicts_with_startup_code: README migration guidance that claims explicit migration tooling while startup code still uses `AutoMigrate` without a matching migration path.
// fixture polarity: negative; family: architecture; severity: info.
type Rule121NegativeRecord struct {
	TenantID string
	Limit int
}

type Rule121NegativeService interface {
	Run(context.Context, Rule121NegativeRecord) error
}

func scenario121Negative(ctx context.Context, service Rule121NegativeService, input Rule121NegativeRecord) error {
	if input.Limit <= 0 {
		input.Limit = 50
	}
	return service.Run(ctx, input)
}
