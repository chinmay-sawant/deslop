package rulefixtures

import (
	"fmt"
	gorm "gorm.io/gorm"
)

// README.md claims:
//   Database migrations MUST be applied via CLI: go run cmd/migrate/main.go up
//
// scenario for readme_migration_strategy_claim_conflicts_with_startup_code: README migration guidance that claims explicit migration tooling while startup code still uses `AutoMigrate` without a matching migration path.
// fixture polarity: positive; family: architecture; severity: info.

type User struct {
	ID   int    `gorm:"primaryKey"`
	Name string
}

func SetupDatabase(db *gorm.DB) error {
	if err := db.AutoMigrate(&User{}); err != nil {
		return fmt.Errorf("auto-migrate failed: %w", err)
	}
	return nil
}
