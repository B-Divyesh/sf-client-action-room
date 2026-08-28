PRAGMA foreign_keys = ON;

CREATE TABLE demo_sessions (
    id TEXT PRIMARY KEY NOT NULL,
    created_at TEXT NOT NULL,
    expires_at TEXT NOT NULL
);

CREATE TABLE demo_actions (
    id TEXT PRIMARY KEY NOT NULL,
    session_id TEXT NOT NULL REFERENCES demo_sessions(id) ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK (kind IN ('approval', 'upload', 'choice', 'external_link')),
    title TEXT NOT NULL CHECK (length(title) BETWEEN 1 AND 120),
    instructions TEXT NOT NULL CHECK (length(instructions) <= 2000),
    due_at TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('open', 'completed')),
    preview_only INTEGER NOT NULL DEFAULT 0 CHECK (preview_only IN (0, 1)),
    version INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL
);
CREATE INDEX demo_actions_by_deadline ON demo_actions(session_id, due_at, id);

CREATE TABLE demo_grants (
    id TEXT PRIMARY KEY NOT NULL,
    token_digest TEXT NOT NULL UNIQUE,
    session_id TEXT NOT NULL REFERENCES demo_sessions(id) ON DELETE CASCADE,
    action_id TEXT NOT NULL REFERENCES demo_actions(id) ON DELETE CASCADE,
    created_at TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    revoked_at TEXT
);
CREATE INDEX demo_grants_by_session ON demo_grants(session_id, action_id);

CREATE TABLE demo_client_sessions (
    id TEXT PRIMARY KEY NOT NULL,
    grant_id TEXT NOT NULL REFERENCES demo_grants(id) ON DELETE CASCADE,
    created_at TEXT NOT NULL,
    expires_at TEXT NOT NULL
);

CREATE TABLE demo_submissions (
    id TEXT PRIMARY KEY NOT NULL,
    session_id TEXT NOT NULL REFERENCES demo_sessions(id) ON DELETE CASCADE,
    action_id TEXT NOT NULL REFERENCES demo_actions(id) ON DELETE CASCADE,
    grant_id TEXT NOT NULL REFERENCES demo_grants(id) ON DELETE CASCADE,
    actor_label TEXT NOT NULL CHECK (length(actor_label) BETWEEN 1 AND 80),
    decision TEXT NOT NULL CHECK (decision IN ('approved', 'changes_requested')),
    comment TEXT NOT NULL DEFAULT '' CHECK (length(comment) <= 1000),
    idempotency_key TEXT NOT NULL,
    request_hash TEXT NOT NULL,
    occurred_at TEXT NOT NULL,
    UNIQUE(grant_id, idempotency_key)
);
CREATE INDEX demo_submissions_by_action ON demo_submissions(action_id, occurred_at);

CREATE TABLE demo_audit_events (
    id TEXT PRIMARY KEY NOT NULL,
    session_id TEXT NOT NULL REFERENCES demo_sessions(id) ON DELETE CASCADE,
    action_id TEXT REFERENCES demo_actions(id) ON DELETE CASCADE,
    event_name TEXT NOT NULL,
    actor_label TEXT NOT NULL,
    decision TEXT,
    occurred_at TEXT NOT NULL
);
CREATE INDEX demo_audit_by_session ON demo_audit_events(session_id, occurred_at, id);
