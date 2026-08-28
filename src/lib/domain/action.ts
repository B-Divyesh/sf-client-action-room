export type ActionKind = 'approval' | 'upload' | 'choice' | 'external_link';
export type ActionStatus = 'open' | 'completed';
export type DisplayStatus = 'open' | 'due-soon' | 'overdue' | 'complete';

export interface DemoAction {
  id: string;
  kind: ActionKind;
  title: string;
  instructions: string;
  due_at: string;
  status: ActionStatus;
  preview_only: boolean;
  version: number;
}

export interface AuditEvent {
  id: string;
  action_id: string | null;
  event_name: string;
  actor_label: string;
  decision: 'approved' | 'changes_requested' | null;
  occurred_at: string;
}

export interface DemoQueue {
  firm: string;
  workspace: string;
  staff_owner: string;
  client_actor: string;
  time_zone: string;
  expires_at: string;
  server_now: string;
  actions: DemoAction[];
  audit: AuditEvent[];
}

export interface Submission {
  id: string;
  actor_label: string;
  decision: 'approved' | 'changes_requested';
  comment: string;
  occurred_at: string;
  replayed: boolean;
}

export interface ClientAction {
  firm: string;
  workspace: string;
  client_actor: string;
  link_expires_at: string;
  action: DemoAction;
  submission: Submission | null;
}

export function displayStatus(action: DemoAction, now = new Date()): DisplayStatus {
  if (action.status === 'completed') return 'complete';
  const due = new Date(action.due_at).getTime();
  const remaining = due - now.getTime();
  if (remaining < 0) return 'overdue';
  if (remaining <= 24 * 60 * 60 * 1000) return 'due-soon';
  return 'open';
}

export function orderedByDeadline(actions: readonly DemoAction[]): DemoAction[] {
  return [...actions].sort((left, right) => {
    const difference = new Date(left.due_at).getTime() - new Date(right.due_at).getTime();
    return difference || left.id.localeCompare(right.id);
  });
}

export function validateApproval(
  actorLabel: string,
  decision: string,
  comment: string,
): string | null {
  if (actorLabel.trim().length === 0) return 'Enter the name that should appear in the record.';
  if (actorLabel.trim().length > 80) return 'Keep the name under 80 characters.';
  if (decision !== 'approved' && decision !== 'changes_requested') {
    return 'Choose approve or ask for changes.';
  }
  if (decision === 'changes_requested' && comment.trim().length === 0) {
    return 'Say what needs to change, then send the answer again.';
  }
  if (comment.length > 1000) return 'Keep the note under 1,000 characters.';
  return null;
}

export function formatEventName(event: AuditEvent): string {
  if (event.event_name === 'action_created') return 'Approval action created';
  if (event.event_name === 'deadline_set') return 'Deadline set';
  if (event.event_name === 'client_link_issued') return 'Client link issued';
  if (event.event_name === 'client_decision_recorded') {
    return event.decision === 'approved' ? 'Approval recorded' : 'Changes requested';
  }
  return 'Action updated';
}
