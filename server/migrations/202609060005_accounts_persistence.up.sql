PRAGMA foreign_keys = ON;

DROP INDEX real_workspace_by_organization;
CREATE INDEX real_workspaces_by_organization
    ON workspaces(organization_id, created_at, id) WHERE namespace = 'real';

ALTER TABLE organizations ADD COLUMN time_zone TEXT NOT NULL DEFAULT 'America/New_York';
ALTER TABLE organizations ADD COLUMN region TEXT NOT NULL DEFAULT 'deployment region';
ALTER TABLE organizations ADD COLUMN retention_days INTEGER NOT NULL DEFAULT 90
    CHECK (retention_days IN (30, 90, 365, 0));
ALTER TABLE organizations ADD COLUMN deletion_requested_at TEXT;
ALTER TABLE organizations ADD COLUMN deletion_due_at TEXT;

CREATE TABLE staff_users (
    oid TEXT PRIMARY KEY NOT NULL,
    display_name TEXT NOT NULL,
    email_snapshot TEXT NOT NULL,
    last_sign_in_at TEXT NOT NULL
);

CREATE TABLE memberships (
    organization_id TEXT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    staff_oid TEXT NOT NULL REFERENCES staff_users(oid) ON DELETE CASCADE,
    role TEXT NOT NULL CHECK (role IN ('owner', 'admin', 'member')),
    accepted_at TEXT NOT NULL,
    PRIMARY KEY (organization_id, staff_oid)
);
CREATE INDEX memberships_by_staff ON memberships(staff_oid, organization_id);

INSERT INTO staff_users (oid, display_name, email_snapshot, last_sign_in_at)
SELECT owner_oid, 'Workspace owner', '', created_at FROM organizations;

INSERT INTO memberships (organization_id, staff_oid, role, accepted_at)
SELECT id, owner_oid, 'owner', created_at FROM organizations;

CREATE TABLE membership_invitations (
    id TEXT PRIMARY KEY NOT NULL,
    organization_id TEXT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    token_digest TEXT NOT NULL UNIQUE,
    role TEXT NOT NULL CHECK (role IN ('admin', 'member')),
    created_by_oid TEXT NOT NULL REFERENCES staff_users(oid),
    created_at TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    accepted_by_oid TEXT REFERENCES staff_users(oid),
    accepted_at TEXT
);

CREATE TABLE subscriptions (
    organization_id TEXT PRIMARY KEY NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    provider_reference TEXT UNIQUE,
    tier TEXT NOT NULL CHECK (tier IN ('starter', 'studio')),
    status TEXT NOT NULL CHECK (status IN ('pending', 'active', 'past_due', 'cancelled', 'expired')),
    period_end TEXT,
    verified_at TEXT NOT NULL,
    provider_event_id TEXT UNIQUE
);

CREATE TABLE idempotency_records (
    organization_id TEXT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    scope TEXT NOT NULL,
    key_digest TEXT NOT NULL,
    request_hash TEXT NOT NULL,
    response_status INTEGER NOT NULL,
    response_body TEXT NOT NULL,
    created_at TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    PRIMARY KEY (organization_id, scope, key_digest)
);

CREATE TABLE organization_exports (
    id TEXT PRIMARY KEY NOT NULL,
    organization_id TEXT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    requested_by_oid TEXT NOT NULL REFERENCES staff_users(oid),
    checksum_sha256 TEXT NOT NULL,
    created_at TEXT NOT NULL,
    expires_at TEXT NOT NULL
);
