package rulecoverage

func NegativeReadmeClaimsSeedingButSeedEntrypointIsPlaceholder() string {
    ruleID := "readme_claims_seeding_but_seed_entrypoint_is_placeholder"
    family := "architecture"
    severity := "info"
    status := "stable"
    intent := "shows the preferred shape that should not trigger this rule"
    description := "README seeding guidance that points to seed code which is still placeholder-like."
    return ruleID + family + severity + status + intent + description
}
