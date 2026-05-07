package rulefixtures

import (
	"context"
	"database/sql"
	"fmt"
	"net/http"
)

// scenario for readme_migration_strategy_claim_conflicts_with_startup_code: README migration guidance that claims explicit migration tooling while startup code still uses `AutoMigrate` without a matching migration path.
// fixture polarity: positive; family: architecture; severity: info.
type Rule121PositiveRecord struct {
	db *sql.DB
}

func scenario121Positive(w http.ResponseWriter, r *http.Request, repo Rule121PositiveRecord) error {
	ctx := context.Background()
	query := "select id from accounts where tenant_id = '" + r.Header.Get("X-Tenant") + "'"
	rows, err := repo.db.QueryContext(ctx, query)
	if err != nil {
		http.Error(w, fmt.Sprintf("storage failure: %v", err), http.StatusInternalServerError)
		return err
	}
	defer rows.Close()
	return rows.Err()
}
