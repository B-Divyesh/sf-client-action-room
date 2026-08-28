export type ComponentStatus = 'planned' | 'scaffolded' | 'built';

export interface ComponentInventoryItem {
  readonly name: string;
  readonly purpose: string;
  readonly states: readonly string[];
  readonly status: ComponentStatus;
}

/** Contract only. M1 implements the components marked for its routes. */
export const componentInventory: readonly ComponentInventoryItem[] = [
  { name: 'ArchiveHeader', purpose: 'Global navigation and account context', states: ['public', 'signed-in', 'menu-open'], status: 'built' },
  { name: 'DemoBanner', purpose: 'Persistent sample-data boundary', states: ['active', 'resetting', 'reset-error'], status: 'built' },
  { name: 'ActionSlip', purpose: 'One accountable request in the deadline queue', states: ['open', 'due-soon', 'overdue', 'complete', 'disabled'], status: 'built' },
  { name: 'DeadlineRail', purpose: 'Groups and orders action slips by deadline', states: ['populated', 'empty', 'loading', 'error'], status: 'built' },
  { name: 'ActionComposer', purpose: 'Creates and edits a client request', states: ['draft', 'invalid', 'saving', 'saved', 'error'], status: 'built' },
  { name: 'ClientWindow', purpose: 'Role-scoped client action surface', states: ['ready', 'submitting', 'complete', 'expired', 'revoked'], status: 'built' },
  { name: 'ApprovalForm', purpose: 'Captures approve or request-changes decisions', states: ['ready', 'invalid', 'submitting', 'complete'], status: 'built' },
  { name: 'UploadTray', purpose: 'Queues files and shows scan outcomes', states: ['empty', 'selected', 'uploading', 'scanning', 'accepted', 'rejected'], status: 'planned' },
  { name: 'ChoiceField', purpose: 'Captures one client selection', states: ['ready', 'invalid', 'submitting', 'complete'], status: 'planned' },
  { name: 'AuditLedger', purpose: 'Shows append-only action history', states: ['populated', 'empty', 'loading', 'error'], status: 'built' },
  { name: 'StatusStamp', purpose: 'Pairs state words with a shape and color', states: ['open', 'due-soon', 'overdue', 'complete', 'expired'], status: 'built' },
  { name: 'ShareLinkPanel', purpose: 'Creates, copies, expires, and revokes client links', states: ['none', 'active', 'copied', 'expired', 'revoked', 'error'], status: 'built' },
  { name: 'ReminderControl', purpose: 'Schedules or sends a transactional reminder', states: ['idle', 'scheduled', 'sending', 'sent', 'error'], status: 'planned' },
  { name: 'InlineNotice', purpose: 'Explains success, warning, and recovery', states: ['info', 'success', 'warning', 'danger'], status: 'built' },
  { name: 'ConfirmDialog', purpose: 'Confirms an irreversible or security-sensitive action', states: ['closed', 'open', 'busy', 'error'], status: 'planned' },
  { name: 'ArchiveSkeleton', purpose: 'Reserves layout during route and queue loading', states: ['queue', 'detail', 'settings'], status: 'built' },
  { name: 'EmptyDocket', purpose: 'Explains an empty queue and its next action', states: ['new-workspace', 'filtered', 'complete'], status: 'built' },
  { name: 'PageShell', purpose: 'Provides skip link, landmarks, route focus, and footer', states: ['public', 'staff', 'client'], status: 'built' }
] as const;
