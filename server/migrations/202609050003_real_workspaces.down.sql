PRAGMA foreign_keys = OFF;

CREATE TABLE staff_workspaces (
    entra_oid TEXT PRIMARY KEY NOT NULL,
    session_id TEXT NOT NULL UNIQUE REFERENCES workspaces(id) ON DELETE CASCADE,
    created_at TEXT NOT NULL
);

INSERT INTO staff_workspaces (entra_oid, session_id, created_at)
SELECT o.owner_oid, w.id, o.created_at
FROM organizations o
JOIN workspaces w ON w.organization_id = o.id AND w.namespace = 'real';

DROP INDEX real_workspace_by_organization;
DROP TABLE organizations;

ALTER TABLE uploads DROP COLUMN scanned_at;
ALTER TABLE uploads DROP COLUMN scan_engine;
ALTER TABLE reminders RENAME COLUMN workspace_id TO session_id;
ALTER TABLE external_visits RENAME COLUMN workspace_id TO session_id;
ALTER TABLE uploads RENAME COLUMN workspace_id TO session_id;
ALTER TABLE choice_submissions RENAME COLUMN workspace_id TO session_id;
ALTER TABLE audit_events RENAME COLUMN workspace_id TO session_id;
ALTER TABLE approval_submissions RENAME COLUMN workspace_id TO session_id;
ALTER TABLE client_grants RENAME COLUMN workspace_id TO session_id;
ALTER TABLE actions RENAME COLUMN workspace_id TO session_id;

ALTER TABLE workspaces DROP COLUMN client_actor;
ALTER TABLE workspaces DROP COLUMN staff_label;
ALTER TABLE workspaces DROP COLUMN client_label;
ALTER TABLE workspaces DROP COLUMN firm_name;
ALTER TABLE workspaces DROP COLUMN organization_id;
ALTER TABLE workspaces DROP COLUMN namespace;

ALTER TABLE reminders RENAME TO demo_reminders;
ALTER TABLE external_visits RENAME TO demo_external_visits;
ALTER TABLE uploads RENAME TO demo_uploads;
ALTER TABLE choice_submissions RENAME TO demo_choice_submissions;
ALTER TABLE external_links RENAME TO demo_external_links;
ALTER TABLE action_options RENAME TO demo_action_options;
ALTER TABLE audit_events RENAME TO demo_audit_events;
ALTER TABLE approval_submissions RENAME TO demo_submissions;
ALTER TABLE client_sessions RENAME TO demo_client_sessions;
ALTER TABLE client_grants RENAME TO demo_grants;
ALTER TABLE actions RENAME TO demo_actions;
ALTER TABLE workspaces RENAME TO demo_sessions;

PRAGMA foreign_keys = ON;
