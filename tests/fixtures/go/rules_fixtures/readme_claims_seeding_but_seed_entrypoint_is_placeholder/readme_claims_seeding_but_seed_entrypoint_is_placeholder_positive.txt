package rulefixtures

import (
	"context"
	"database/sql"
	"fmt"
	"net/http"
)

// scenario for readme_claims_seeding_but_seed_entrypoint_is_placeholder: README seeding guidance that points to seed code which is still placeholder-like.
// fixture polarity: positive; family: architecture; severity: info.
type Rule120PositiveRecord struct {
	db *sql.DB
}

func scenario120Positive(w http.ResponseWriter, r *http.Request, repo Rule120PositiveRecord) error {
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
