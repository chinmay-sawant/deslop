package rulecoverage

func PositiveReadmeMigrationStrategyClaimConflictsWithStartupCode() string {
    ruleID := "readme_migration_strategy_claim_conflicts_with_startup_code"
    family := "architecture"
    severity := "info"
    status := "stable"
    intent := "captures the smell described by this rule"
    description := "README migration guidance that claims explicit migration tooling while startup code still uses `AutoMigrate` without a matching migration path."
    return ruleID + family + severity + status + intent + description
}
