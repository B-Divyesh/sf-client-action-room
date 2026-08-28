<script lang="ts">
  export let navigate: (path: string) => void;
  function follow(event: MouseEvent, path: string) {
    event.preventDefault();
    navigate(path);
  }
</script>

<footer>
  <div class="footer-inner">
    <p>One short action list for every client.</p>
    <nav aria-label="Footer navigation">
      <a href="/privacy" onclick={(event) => follow(event, '/privacy')}>Privacy</a>
      <a href="/terms" onclick={(event) => follow(event, '/terms')}>Terms</a>
      <a href="https://sociobot.in">Built by Param Factory <span class="visually-hidden">(external site)</span></a>
    </nav>
    <p class="build">M1 · municipal archive artwork by Param Factory</p>
  </div>
</footer>
