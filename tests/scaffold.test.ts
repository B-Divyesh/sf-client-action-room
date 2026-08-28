import { describe, expect, it } from 'vitest';

import { componentInventory } from '../src/lib/components/inventory';

describe('planning scaffold', () => {
  it('keeps the component contract within the venture inventory range', () => {
    expect(componentInventory.length).toBeGreaterThanOrEqual(12);
    expect(componentInventory.length).toBeLessThanOrEqual(20);
  });

  it('gives each planned component distinct states', () => {
    const names = componentInventory.map(({ name }) => name);
    expect(new Set(names).size).toBe(names.length);

    for (const item of componentInventory) {
      expect(item.states.length).toBeGreaterThan(0);
      expect(new Set(item.states).size).toBe(item.states.length);
      expect(item.status).toBe('planned');
    }
  });
});
