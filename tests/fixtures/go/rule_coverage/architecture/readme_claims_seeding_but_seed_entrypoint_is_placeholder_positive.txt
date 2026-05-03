package seed

import (
	"gorm.io/gorm"
)

// Positive scenario for readme_claims_seeding_but_seed_entrypoint_is_placeholder.
// Rule intent: README seeding guidance that points to seed code which is still placeholder-like.
// Anti-pattern: Seed function panics with "not implemented" despite being referenced in docs.

// README.md says:
//   ## Seeding the Database
//   Run `go run cmd/seed/main.go` to populate default data.
//   This will create admin users, default roles, and sample products.

type AdminUser struct {
	ID       uint   `gorm:"primaryKey"`
	Username string `gorm:"column:username"`
	Email    string `gorm:"column:email"`
}

func SeedAdminUsers(db *gorm.DB) error {
	panic("not implemented")
}

func SeedDefaultRoles(db *gorm.DB) error {
	panic("not implemented")
}

func SeedSampleProducts(db *gorm.DB) error {
	panic("not implemented")
}

func RunAll(db *gorm.DB) error {
	if err := SeedAdminUsers(db); err != nil {
		return err
	}
	if err := SeedDefaultRoles(db); err != nil {
		return err
	}
	return SeedSampleProducts(db)
}
