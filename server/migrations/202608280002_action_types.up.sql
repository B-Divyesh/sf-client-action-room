CREATE TABLE demo_action_options (
    action_id TEXT NOT NULL REFERENCES demo_actions(id) ON DELETE CASCADE,
    option_key TEXT NOT NULL,
    label TEXT NOT NULL CHECK (length(label) BETWEEN 1 AND 120),
    position INTEGER NOT NULL,
    PRIMARY KEY (action_id, option_key)
);

CREATE TABLE demo_external_links (
    action_id TEXT PRIMARY KEY NOT NULL REFERENCES demo_actions(id) ON DELETE CASCADE,
    url TEXT NOT NULL CHECK (url LIKE 'https://%'),
    destination_host TEXT NOT NULL
);

CREATE TABLE demo_choice_submissions (
    id TEXT PRIMARY KEY NOT NULL,
    session_id TEXT NOT NULL REFERENCES demo_sessions(id) ON DELETE CASCADE,
    action_id TEXT NOT NULL REFERENCES demo_actions(id) ON DELETE CASCADE,
    grant_id TEXT NOT NULL REFERENCES demo_grants(id) ON DELETE CASCADE,
    actor_label TEXT NOT NULL CHECK (length(actor_label) BETWEEN 1 AND 80),
    option_key TEXT NOT NULL,
    option_label TEXT NOT NULL,
    occurred_at TEXT NOT NULL,
    UNIQUE(grant_id)
);

CREATE TABLE demo_uploads (
    id TEXT PRIMARY KEY NOT NULL,
    session_id TEXT NOT NULL REFERENCES demo_sessions(id) ON DELETE CASCADE,
    action_id TEXT NOT NULL REFERENCES demo_actions(id) ON DELETE CASCADE,
    grant_id TEXT NOT NULL REFERENCES demo_grants(id) ON DELETE CASCADE,
    actor_label TEXT NOT NULL CHECK (length(actor_label) BETWEEN 1 AND 80),
    original_filename TEXT NOT NULL,
    detected_mime TEXT NOT NULL,
    byte_size INTEGER NOT NULL CHECK (byte_size BETWEEN 1 AND 5242880),
    checksum_sha256 TEXT NOT NULL,
    scan_state TEXT NOT NULL CHECK (scan_state IN ('clean', 'rejected')),
    content BLOB,
    expires_at TEXT NOT NULL,
    occurred_at TEXT NOT NULL,
    UNIQUE(grant_id)
);

CREATE TABLE demo_external_visits (
    id TEXT PRIMARY KEY NOT NULL,
    session_id TEXT NOT NULL REFERENCES demo_sessions(id) ON DELETE CASCADE,
    action_id TEXT NOT NULL REFERENCES demo_actions(id) ON DELETE CASCADE,
    grant_id TEXT NOT NULL REFERENCES demo_grants(id) ON DELETE CASCADE,
    actor_label TEXT NOT NULL CHECK (length(actor_label) BETWEEN 1 AND 80),
    destination_host TEXT NOT NULL,
    occurred_at TEXT NOT NULL,
    UNIQUE(grant_id)
);

CREATE TABLE demo_reminders (
    id TEXT PRIMARY KEY NOT NULL,
    session_id TEXT NOT NULL REFERENCES demo_sessions(id) ON DELETE CASCADE,
    action_id TEXT NOT NULL REFERENCES demo_actions(id) ON DELETE CASCADE,
    scheduled_for TEXT NOT NULL,
    channel TEXT NOT NULL CHECK (channel = 'email'),
    status TEXT NOT NULL CHECK (status = 'scheduled'),
    created_at TEXT NOT NULL,
    UNIQUE(session_id, action_id)
);

CREATE TABLE staff_workspaces (
    entra_oid TEXT PRIMARY KEY NOT NULL,
    session_id TEXT NOT NULL UNIQUE REFERENCES demo_sessions(id) ON DELETE CASCADE,
    created_at TEXT NOT NULL
);
