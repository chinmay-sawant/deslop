package rulecoverage

func NegativeReadmeMigrationStrategyClaimConflictsWithStartupCode() string {
    ruleID := "readme_migration_strategy_claim_conflicts_with_startup_code"
    family := "architecture"
    severity := "info"
    status := "stable"
    intent := "shows the preferred shape that should not trigger this rule"
    description := "README migration guidance that claims explicit migration tooling while startup code still uses `AutoMigrate` without a matching migration path."
    return ruleID + family + severity + status + intent + description
}
