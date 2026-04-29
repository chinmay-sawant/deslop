package rulecoverage

func PositiveReadmeClaimsSeedingButSeedEntrypointIsPlaceholder() string {
    ruleID := "readme_claims_seeding_but_seed_entrypoint_is_placeholder"
    family := "architecture"
    severity := "info"
    status := "stable"
    intent := "captures the smell described by this rule"
    description := "README seeding guidance that points to seed code which is still placeholder-like."
    return ruleID + family + severity + status + intent + description
}
