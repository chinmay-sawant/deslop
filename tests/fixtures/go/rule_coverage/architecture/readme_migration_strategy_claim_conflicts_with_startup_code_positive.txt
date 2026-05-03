package main

import (
	"log"

	"gorm.io/driver/postgres"
	"gorm.io/gorm"
)

// Positive scenario for readme_migration_strategy_claim_conflicts_with_startup_code.
// Rule intent: README migration guidance that claims explicit migration tooling while
// startup code still uses AutoMigrate without a matching migration path.
// Anti-pattern: README says "run migrations manually with goose" but startup code auto-migrates.

// README.md says:
//   ## Database Migrations
//   We use goose for versioned migrations. Run them manually before deploying:
//     goose -dir migrations up
//   DO NOT rely on automatic migration at startup.

type Invoice struct {
	ID        uint    `gorm:"primaryKey"`
	Number    string  `gorm:"column:number"`
	Amount    float64 `gorm:"column:amount"`
	Currency  string  `gorm:"column:currency"`
}

func startApp(dsn string) {
	db, err := gorm.Open(postgres.Open(dsn), &gorm.Config{})
	if err != nil {
		log.Fatalf("failed to connect: %v", err)
	}

	// Conflicts with README: auto-migrating despite docs saying "use goose manually"
	if err := db.AutoMigrate(&Invoice{}); err != nil {
		log.Fatalf("auto-migrate failed: %v", err)
	}

	log.Println("server starting...")
}

func main() {
	startApp("postgres://localhost:5432/invoicing")
}
