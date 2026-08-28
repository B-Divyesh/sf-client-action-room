<script lang="ts">
  export let busy = false;
  export let onReset: () => void;
  export let onStart: () => void;
</script>

<aside class="demo-banner" aria-label="Demo mode">
  <div>
    <strong>Demo — sample data, nothing is saved</strong>
    <span>Changes stay in this temporary room.</span>
  </div>
  <div class="banner-actions">
    <button class="text-button" type="button" disabled={busy} onclick={onReset}>{busy ? 'Resetting…' : 'Reset demo'}</button>
    <button class="text-button" type="button" disabled={busy} onclick={onStart}>Start for real</button>
  </div>
</aside>
