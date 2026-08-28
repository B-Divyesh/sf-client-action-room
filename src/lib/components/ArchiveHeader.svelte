<script lang="ts">
  export let route: string;
  export let navigate: (path: string) => void;
  export let theme: 'light' | 'dark';
  export let toggleTheme: () => void;

  function follow(event: MouseEvent, path: string) {
    if (event.button !== 0 || event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return;
    event.preventDefault();
    navigate(path);
  }
</script>

<header class:client-header={route === 'client'}>
  <div class="header-inner">
    <a class="wordmark" href="/" onclick={(event) => follow(event, '/')}>Client Action Room</a>
    {#if route !== 'client'}
      <nav aria-label="Main navigation">
        <a href="/demo" aria-current={route === 'demo' ? 'page' : undefined} onclick={(event) => follow(event, '/demo')}>Demo</a>
        <a href="/#how-it-works">How it works</a>
        <a href="/privacy" aria-current={route === 'privacy' ? 'page' : undefined} onclick={(event) => follow(event, '/privacy')}>Privacy</a>
      </nav>
    {:else}
      <span class="client-label">Focused client request</span>
    {/if}
    <button class="theme-button" type="button" onclick={toggleTheme} aria-label={`Use ${theme === 'light' ? 'dark' : 'light'} theme`}>
      <span aria-hidden="true">{theme === 'light' ? '◐' : '◑'}</span>
    </button>
  </div>
</header>
