package rulecoverage

import (
    "context"
    "database/sql"
    "net/http"
)

// Negative scenario for readme_claims_seeding_but_seed_entrypoint_is_placeholder: shows the preferred shape for this rule.
// Rule intent: README seeding guidance that points to seed code which is still placeholder-like.
type User struct {
    ID string
    Status string
}

type UserRequest struct {
    Status *string `json:"status"`
}

type UserResponse struct {
    ID string `json:"id"`
    Status string `json:"status"`
}

type UserRepository interface {
    FindByID(ctx context.Context, id string) (User, error)
}

type UserService struct { repo UserRepository }

func (s UserService) NegativeReadmeClaimsSeedingButSeedEntrypointIsPlaceholder(ctx context.Context, id string, req UserRequest) (UserResponse, error) {
    user, err := s.repo.FindByID(ctx, id)
    if err != nil {
        return UserResponse{}, err
    }
    if req.Status != nil {
        user.Status = *req.Status
    }
    return UserResponse{ID: user.ID, Status: user.Status}, nil
}

func writeUser(w http.ResponseWriter, response UserResponse) {
    w.Header().Set("Content-Type", "application/json")
    w.WriteHeader(http.StatusOK)
}

func queryUser(ctx context.Context, db *sql.DB, id string) (*sql.Rows, error) {
    return db.QueryContext(ctx, "SELECT id, status FROM users WHERE id = ?", id)
}
