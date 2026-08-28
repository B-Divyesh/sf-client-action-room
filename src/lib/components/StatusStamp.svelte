<script lang="ts">
  import type { DisplayStatus } from '../domain/action';
  export let status: DisplayStatus;
  const labels: Record<DisplayStatus, string> = {
    open: 'Open',
    'due-soon': 'Due soon',
    overdue: 'Overdue',
    complete: 'Complete',
  };
</script>

<span class="status-stamp status-{status}"><span aria-hidden="true" class="status-shape"></span>{labels[status]}</span>
