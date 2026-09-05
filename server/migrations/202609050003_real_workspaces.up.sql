PRAGMA foreign_keys = OFF;

ALTER TABLE demo_sessions RENAME TO workspaces;
ALTER TABLE demo_actions RENAME TO actions;
ALTER TABLE demo_grants RENAME TO client_grants;
ALTER TABLE demo_client_sessions RENAME TO client_sessions;
ALTER TABLE demo_submissions RENAME TO approval_submissions;
ALTER TABLE demo_audit_events RENAME TO audit_events;
ALTER TABLE demo_action_options RENAME TO action_options;
ALTER TABLE demo_external_links RENAME TO external_links;
ALTER TABLE demo_choice_submissions RENAME TO choice_submissions;
ALTER TABLE demo_uploads RENAME TO uploads;
ALTER TABLE demo_external_visits RENAME TO external_visits;
ALTER TABLE demo_reminders RENAME TO reminders;

ALTER TABLE workspaces ADD COLUMN namespace TEXT NOT NULL DEFAULT 'demo'
    CHECK (namespace IN ('demo', 'real'));
ALTER TABLE workspaces ADD COLUMN organization_id TEXT;
ALTER TABLE workspaces ADD COLUMN firm_name TEXT NOT NULL DEFAULT 'Northline Studio';
ALTER TABLE workspaces ADD COLUMN client_label TEXT NOT NULL DEFAULT 'Alder Street Bakery launch';
ALTER TABLE workspaces ADD COLUMN staff_label TEXT NOT NULL DEFAULT 'Theo Grant';
ALTER TABLE workspaces ADD COLUMN client_actor TEXT NOT NULL DEFAULT 'Maya Chen';

ALTER TABLE actions RENAME COLUMN session_id TO workspace_id;
ALTER TABLE client_grants RENAME COLUMN session_id TO workspace_id;
ALTER TABLE approval_submissions RENAME COLUMN session_id TO workspace_id;
ALTER TABLE audit_events RENAME COLUMN session_id TO workspace_id;
ALTER TABLE choice_submissions RENAME COLUMN session_id TO workspace_id;
ALTER TABLE uploads RENAME COLUMN session_id TO workspace_id;
ALTER TABLE external_visits RENAME COLUMN session_id TO workspace_id;
ALTER TABLE reminders RENAME COLUMN session_id TO workspace_id;
ALTER TABLE uploads ADD COLUMN scan_engine TEXT;
ALTER TABLE uploads ADD COLUMN scanned_at TEXT;

CREATE TABLE organizations (
    id TEXT PRIMARY KEY NOT NULL,
    owner_oid TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL CHECK (length(name) BETWEEN 1 AND 80),
    created_at TEXT NOT NULL
);

CREATE UNIQUE INDEX real_workspace_by_organization
    ON workspaces(organization_id) WHERE namespace = 'real';

DROP TABLE staff_workspaces;

PRAGMA foreign_keys = ON;
