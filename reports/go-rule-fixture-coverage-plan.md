# Go Rule Fixture Coverage Plan

Current baseline:

- Go catalog total: 753 rules.
- Integration assertions with positive coverage: 201 rules.
- Integration assertions with negative coverage: 154 rules.
- Integration assertions with both positive and negative coverage: 153 rules.
- Missing at least one behavioral integration assertion side: 600 rules.

Fixture-file plan:

1. Add a deterministic generated fixture location at `tests/fixtures/go/rule_coverage/<family>/`.
2. Create `<rule_id>_positive.txt` and `<rule_id>_negative.txt` for every Go catalog rule.
3. Keep each fixture as parseable Go source text carrying the rule id, family, severity, status, description, and polarity intent.
4. Add an integration guard that fails when any Go catalog rule is missing either fixture file.
5. Extend the Go coverage audit script so it reports both behavioral assertion coverage and fixture-file pair coverage.
6. Continue converting generated fixture pairs into behavior-specific scanner assertions family by family, starting with the 600 rules missing at least one assertion side.
