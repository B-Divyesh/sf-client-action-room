import { describe, expect, it } from 'vitest';

import { componentInventory } from '../src/lib/components/inventory';
import {
  displayStatus,
  orderedByDeadline,
  validateApproval,
  type DemoAction,
} from '../src/lib/domain/action';
import { resolveRoute, routeMeta } from '../src/lib/routes/routes';

const action = (id: string, due_at: string, status: DemoAction['status'] = 'open'): DemoAction => ({
  id,
  due_at,
  status,
  kind: 'approval',
  title: `Action ${id}`,
  instructions: 'Review this proof.',
  preview_only: false,
  version: 1,
});

describe('action state and deadlines', () => {
  it('orders requests by the absolute deadline across date boundaries', () => {
    const actions = [
      action('later', '2026-11-01T04:30:00Z'),
      action('earlier', '2026-11-01T03:30:00Z'),
      action('first', '2026-10-31T23:30:00Z'),
    ];
    expect(orderedByDeadline(actions).map(({ id }) => id)).toEqual(['first', 'earlier', 'later']);
  });

  it('derives overdue, due-soon, open, and complete states', () => {
    const now = new Date('2026-08-28T14:00:00Z');
    expect(displayStatus(action('a', '2026-08-28T13:00:00Z'), now)).toBe('overdue');
    expect(displayStatus(action('b', '2026-08-29T13:00:00Z'), now)).toBe('due-soon');
    expect(displayStatus(action('c', '2026-08-30T14:00:00Z'), now)).toBe('open');
    expect(displayStatus(action('d', '2026-08-28T13:00:00Z', 'completed'), now)).toBe('complete');
  });
});

describe('plain approval validation', () => {
  it('requires the actor, decision, and a note for changes', () => {
    expect(validateApproval('', '', '')).toContain('name');
    expect(validateApproval('Maya Chen', '', '')).toContain('Choose');
    expect(validateApproval('Maya Chen', 'changes_requested', '')).toContain('needs to change');
    expect(validateApproval('Maya Chen', 'approved', '')).toBeNull();
  });
});

describe('routing and components', () => {
  it('maps each public route to a distinct plain title', () => {
    expect(resolveRoute('/', '?demo=1')).toBe('demo');
    expect(resolveRoute('/privacy')).toBe('privacy');
    expect(resolveRoute('/missing')).toBe('not-found');
    expect(new Set(Object.values(routeMeta).map(({ title }) => title)).size).toBe(
      Object.keys(routeMeta).length,
    );
  });

  it('keeps the venture component inventory explicit', () => {
    expect(componentInventory).toHaveLength(18);
    expect(componentInventory.filter(({ status }) => status === 'built').length).toBeGreaterThan(10);
  });
});
