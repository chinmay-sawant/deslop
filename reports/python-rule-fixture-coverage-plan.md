# Python Rule Fixture Coverage Plan

Current baseline:

- Python catalog total: 691 rules.
- Integration assertions with positive coverage: 125 rules.
- Integration assertions with negative coverage: 82 rules.
- Integration assertions with both positive and negative coverage: 79 rules.
- Missing at least one behavioral integration assertion side: 612 rules.

Fixture-file plan:

1. Add a deterministic generated fixture location at `tests/fixtures/python/rule_coverage/<family>/`.
2. Create `<rule_id>_positive.txt` and `<rule_id>_negative.txt` for every Python catalog rule.
3. Keep each fixture as parseable Python source text carrying the rule id, family, severity, status, description, and polarity intent.
4. Add an integration guard that fails when any Python catalog rule is missing either fixture file.
5. Add a Python coverage audit script so it reports both behavioral assertion coverage and fixture-file pair coverage.
6. Continue converting generated fixture pairs into behavior-specific scanner assertions family by family, starting with the 612 rules missing at least one assertion side.
