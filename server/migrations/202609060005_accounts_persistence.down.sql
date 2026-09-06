PRAGMA foreign_keys = ON;

DROP TABLE IF EXISTS organization_exports;
DROP TABLE IF EXISTS idempotency_records;
DROP TABLE IF EXISTS subscriptions;
DROP TABLE IF EXISTS membership_invitations;
DROP TABLE IF EXISTS memberships;
DROP TABLE IF EXISTS staff_users;

ALTER TABLE organizations DROP COLUMN deletion_due_at;
ALTER TABLE organizations DROP COLUMN deletion_requested_at;
ALTER TABLE organizations DROP COLUMN retention_days;
ALTER TABLE organizations DROP COLUMN region;
ALTER TABLE organizations DROP COLUMN time_zone;

DROP INDEX real_workspaces_by_organization;
CREATE UNIQUE INDEX real_workspace_by_organization
    ON workspaces(organization_id) WHERE namespace = 'real';
