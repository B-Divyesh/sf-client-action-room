-- Email delivery is intentionally unavailable in demo namespaces. Keeping a
-- separate queue table makes the no-delivery boundary observable and leaves a
-- clear handoff point for the later transactional-email milestone.
CREATE TABLE email_delivery_queue (
    id TEXT PRIMARY KEY NOT NULL,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    reminder_id TEXT NOT NULL REFERENCES reminders(id) ON DELETE CASCADE,
    created_at TEXT NOT NULL
);

CREATE INDEX email_delivery_queue_by_workspace
    ON email_delivery_queue(workspace_id, created_at);
