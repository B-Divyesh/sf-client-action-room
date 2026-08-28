<script lang="ts">
  import { onMount, tick } from 'svelte';
  import ArchiveHeader from './lib/components/ArchiveHeader.svelte';
  import DemoBanner from './lib/components/DemoBanner.svelte';
  import SiteFooter from './lib/components/SiteFooter.svelte';
  import StatusStamp from './lib/components/StatusStamp.svelte';
  import { api, ApiError } from './lib/api';
  import { beginStaffSignIn, finishStaffSignIn, signOut, staffToken } from './lib/auth';
  import type { AccountInfo } from '@azure/msal-browser';
  import {
    displayStatus,
    formatEventName,
    orderedByDeadline,
    validateApproval,
    type ClientAction,
    type Completion,
    type DemoAction,
    type DemoQueue,
    type Submission,
  } from './lib/domain/action';
  import { resolveRoute, routeMeta, type RouteName } from './lib/routes/routes';

  const canonicalOrigin = 'https://client-action-room.sociobot.in';

  let pathname = window.location.pathname;
  let search = window.location.search;
  let route: RouteName = resolveRoute(pathname, search);
  let demo: DemoQueue | null = null;
  let client: ClientAction | null = null;
  let loading = false;
  let busy = false;
  let notice = '';
  let error = '';
  let requestId = '';
  let publishedLink: { path: string; expires_at: string } | null = null;
  let expiredLink: { path: string; expires_at: string } | null = null;
  let clientExpired = false;
  let actorLabel = 'Maya Chen';
  let decision = '';
  let comment = '';
  let submission: Submission | null = null;
  let completion: Completion | null = null;
  let selectedChoice = '';
  let selectedFile: File | null = null;
  let formError = '';
  let newTitle = '';
  let newInstructions = '';
  let newDeadline = '';
  let composerError = '';
  let theme: 'light' | 'dark' = 'light';
  let staffAccount: AccountInfo | null = null;
  let staffProfile: { id: string; name: string; email: string } | null = null;

  $: meta = routeMeta[route];
  $: canonical = `${canonicalOrigin}${meta.canonicalPath}`;
  $: sortedActions = demo ? orderedByDeadline(demo.actions) : [];

  onMount(() => {
    theme = document.documentElement.dataset.theme === 'dark' ? 'dark' : 'light';
    const pop = () => {
      pathname = window.location.pathname;
      search = window.location.search;
      route = resolveRoute(pathname, search);
      void loadRoute();
    };
    window.addEventListener('popstate', pop);
    void loadRoute();
    return () => window.removeEventListener('popstate', pop);
  });

  function navigate(target: string) {
    window.history.pushState({}, '', target);
    pathname = window.location.pathname;
    search = window.location.search;
    route = resolveRoute(pathname, search);
    void loadRoute();
  }

  async function loadRoute() {
    error = '';
    notice = '';
    requestId = '';
    if (route === 'demo') await loadDemo(new URLSearchParams(search).get('reset') === '1');
    if (route === 'client') await loadClient();
    if (route === 'workspace' || route === 'auth-callback') await loadWorkspace();
    if (route === 'home' && new URLSearchParams(search).get('start') === '1') {
      notice = 'The sample room was cleared. Real accounts and monthly plans are not available in this release.';
    }
    await tick();
    document.querySelector<HTMLElement>('main h1')?.focus({ preventScroll: true });
    window.scrollTo({ top: 0, behavior: 'instant' });
  }

  async function loadDemo(forceReset = false) {
    loading = true;
    publishedLink = null;
    expiredLink = null;
    try {
      if (forceReset) {
        demo = await api<DemoQueue>('/api/v1/demo/session/reset', {
          method: 'POST',
          headers: { 'Idempotency-Key': crypto.randomUUID() },
        });
        window.history.replaceState({}, '', '/demo');
        pathname = '/demo';
        search = '';
        notice = 'The demo is back to its original sample.';
      } else {
        demo = await api<DemoQueue>('/api/v1/demo/session/ensure', {
          method: 'POST',
          headers: { 'Idempotency-Key': crypto.randomUUID() },
        });
      }
    } catch (caught) {
      showError(caught);
    } finally {
      loading = false;
    }
  }

  async function resetDemo() {
    busy = true;
    error = '';
    try {
      demo = await api<DemoQueue>('/api/v1/demo/session/reset', {
        method: 'POST',
        headers: { 'Idempotency-Key': crypto.randomUUID() },
      });
      publishedLink = null;
      expiredLink = null;
      notice = 'The demo is back to its original sample.';
    } catch (caught) {
      showError(caught);
    } finally {
      busy = false;
    }
  }

  async function startForReal() {
    busy = true;
    try {
      await api<void>('/api/v1/demo/session', { method: 'DELETE' });
    } catch {
      // The next screen still explains the real-account boundary.
    }
    try {
      await beginStaffSignIn();
    } catch (caught) {
      busy = false;
      showError(caught);
      navigate('/workspace');
    }
  }

  async function loadWorkspace() {
    loading = true;
    try {
      staffAccount = await finishStaffSignIn();
      if (route === 'auth-callback') {
        navigate('/workspace');
        return;
      }
      if (staffAccount) {
        const token = await staffToken(staffAccount);
        staffProfile = await api<{ id: string; name: string; email: string }>('/api/v1/me', {
          headers: { Authorization: `Bearer ${token}` },
        });
        demo = await api<DemoQueue>('/api/v1/demo/queue');
      }
    } catch (caught) { showError(caught); }
    finally { loading = false; }
  }

  async function leaveWorkspace() {
    if (staffAccount) await signOut(staffAccount);
  }

  async function publish(action: DemoAction) {
    busy = true;
    error = '';
    try {
      publishedLink = await api<{ path: string; expires_at: string }>(
        `/api/v1/demo/actions/${encodeURIComponent(action.id)}/publish`,
        { method: 'POST', headers: { 'Idempotency-Key': crypto.randomUUID() } },
      );
      demo = await api<DemoQueue>('/api/v1/demo/queue');
      notice = 'The client link is ready. It can open only this approval.';
    } catch (caught) {
      showError(caught);
    } finally {
      busy = false;
    }
  }

  async function makeExpiredLink() {
    busy = true;
    error = '';
    try {
      expiredLink = await api<{ path: string; expires_at: string }>(
        '/api/v1/demo/client-links/expired',
        { method: 'POST', headers: { 'Idempotency-Key': crypto.randomUUID() } },
      );
    } catch (caught) {
      showError(caught);
    } finally {
      busy = false;
    }
  }

  async function createApproval(event: SubmitEvent) {
    event.preventDefault();
    composerError = '';
    if (!newTitle.trim() || !newInstructions.trim() || !newDeadline) {
      composerError = 'Name the approval, explain what to review, and choose a deadline.';
      return;
    }
    busy = true;
    try {
      await api<DemoAction>('/api/v1/demo/actions', {
        method: 'POST',
        headers: { 'Idempotency-Key': crypto.randomUUID() },
        body: JSON.stringify({
          title: newTitle,
          instructions: newInstructions,
          due_at: new Date(newDeadline).toISOString(),
        }),
      });
      demo = await api<DemoQueue>('/api/v1/demo/queue');
      newTitle = '';
      newInstructions = '';
      newDeadline = '';
      notice = 'The new approval is in the deadline list.';
    } catch (caught) {
      composerError = caught instanceof Error ? caught.message : 'We could not create the approval. Try again.';
    } finally {
      busy = false;
    }
  }

  async function copyClientLink() {
    if (!publishedLink) return;
    await navigator.clipboard.writeText(new URL(publishedLink.path, window.location.origin).toString());
    notice = 'Client link copied.';
  }

  async function loadClient() {
    loading = true;
    client = null;
    submission = null;
    completion = null;
    clientExpired = false;
    try {
      const token = new URLSearchParams(window.location.hash.slice(1)).get('access');
      if (token) {
        await api<{ exchanged: boolean }>('/api/v1/client-links/exchange', {
          method: 'POST',
          body: JSON.stringify({ token }),
        });
        window.history.replaceState({}, '', '/client');
      }
      client = await api<ClientAction>('/api/v1/client/actions');
      submission = client.submission;
      actorLabel = client.client_actor;
    } catch (caught) {
      if (caught instanceof ApiError && [401, 410].includes(caught.status)) {
        clientExpired = true;
        error = caught.message;
      } else {
        showError(caught);
      }
    } finally {
      loading = false;
    }
  }

  async function submitApproval(event: SubmitEvent) {
    event.preventDefault();
    formError = validateApproval(actorLabel, decision, comment) ?? '';
    if (formError) {
      await tick();
      document.getElementById('approval-error')?.focus();
      return;
    }
    if (!client) return;
    busy = true;
    try {
      submission = await api<Submission>(
        `/api/v1/client/actions/${encodeURIComponent(client.action.id)}/submissions`,
        {
          method: 'POST',
          headers: { 'Idempotency-Key': crypto.randomUUID() },
          body: JSON.stringify({ actor_label: actorLabel, decision, comment }),
        },
      );
      notice = 'Your answer is recorded.';
    } catch (caught) {
      formError = caught instanceof Error ? caught.message : 'We could not record your answer. Try again.';
    } finally {
      busy = false;
    }
  }

  async function submitChoice(event: SubmitEvent) {
    event.preventDefault();
    if (!client || !selectedChoice) { formError = 'Choose one photo crop.'; return; }
    busy = true; formError = '';
    try {
      completion = await api<Completion>(`/api/v1/client/actions/${encodeURIComponent(client.action.id)}/choice`, {
        method: 'POST', body: JSON.stringify({ actor_label: actorLabel, option_key: selectedChoice }),
      });
      notice = 'Your choice is recorded.';
    } catch (caught) { formError = caught instanceof Error ? caught.message : 'We could not record your choice. Try again.'; }
    finally { busy = false; }
  }

  async function submitUpload(event: SubmitEvent) {
    event.preventDefault();
    if (!client || !selectedFile) { formError = 'Choose one PDF file under 5 MB.'; return; }
    const form = new FormData();
    form.set('actor_label', actorLabel);
    form.set('file', selectedFile);
    busy = true; formError = '';
    try {
      completion = await api<Completion>(`/api/v1/client/actions/${encodeURIComponent(client.action.id)}/upload`, { method: 'POST', body: form });
      notice = 'Your file passed the safety scan and is recorded.';
    } catch (caught) { formError = caught instanceof Error ? caught.message : 'We could not scan your file. Try again.'; }
    finally { busy = false; }
  }

  async function openExternal() {
    if (!client) return;
    busy = true; formError = '';
    try {
      completion = await api<Completion>(`/api/v1/client/actions/${encodeURIComponent(client.action.id)}/visit`, {
        method: 'POST', body: JSON.stringify({ actor_label: actorLabel }),
      });
      notice = 'The destination is ready. This records only that you opened it.';
    } catch (caught) { formError = caught instanceof Error ? caught.message : 'We could not open this link. Try again.'; }
    finally { busy = false; }
  }

  async function scheduleReminder(action: DemoAction) {
    busy = true; error = '';
    try {
      const result = await api<{ scheduled_for: string }>(`/api/v1/demo/actions/${encodeURIComponent(action.id)}/reminder`, { method: 'POST' });
      demo = await api<DemoQueue>('/api/v1/demo/queue');
      notice = `Reminder scheduled for ${formatDate(result.scheduled_for)}.`;
    } catch (caught) { showError(caught); }
    finally { busy = false; }
  }

  function showError(caught: unknown) {
    if (caught instanceof ApiError) {
      error = caught.message;
      requestId = caught.requestId ?? '';
    } else {
      error = 'We could not load this page. Check your connection and try again.';
    }
  }

  function toggleTheme() {
    theme = theme === 'light' ? 'dark' : 'light';
    document.documentElement.dataset.theme = theme;
    localStorage.setItem('car:theme', theme);
  }

  function skipToMain(event: MouseEvent) {
    event.preventDefault();
    const main = document.getElementById('main');
    main?.focus();
    main?.scrollIntoView();
  }

  function formatDate(value: string, includeTime = true) {
    return new Intl.DateTimeFormat('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
      ...(includeTime ? { hour: 'numeric', minute: '2-digit' } : {}),
      timeZone: 'America/New_York',
      timeZoneName: includeTime ? 'short' : undefined,
    }).format(new Date(value));
  }

  function dueLabel(action: DemoAction, serverNow?: string) {
    const status = displayStatus(action, serverNow ? new Date(serverNow) : new Date());
    if (status === 'overdue') return `Overdue · ${formatDate(action.due_at, false)}`;
    return `Due ${formatDate(action.due_at, false)}`;
  }
</script>

<svelte:head>
  <title>{meta.title}</title>
  <meta name="description" content={meta.description} />
  <link rel="canonical" href={canonical} />
  <meta property="og:title" content={meta.title} />
  <meta property="og:description" content={meta.description} />
  <meta property="og:url" content={canonical} />
  <meta name="twitter:title" content={meta.title} />
  <meta name="twitter:description" content={meta.description} />
</svelte:head>

<a class="skip-link" href="#main" onclick={skipToMain}>Skip to main content</a>
<div class="route-announcer visually-hidden" aria-live="polite">{meta.title}</div>
{#if route === 'demo' || route === 'client'}
  <DemoBanner busy={busy} onReset={() => navigate('/demo?reset=1')} onStart={startForReal} />
{/if}
<ArchiveHeader route={route} {navigate} {theme} {toggleTheme} />

{#if route === 'home'}
  <main id="main" class="landing" tabindex="-1">
    <section class="hero" aria-labelledby="home-title">
      <div class="hero-copy">
        <p class="eyebrow">One clear docket for the client</p>
        <h1 id="home-title" tabindex="-1">Get client actions done on time</h1>
        <p class="lede">For small firms chasing approvals, files, choices, and payment links across email.</p>
        <div class="primary-row">
          <a class="button primary" href="/demo" onclick={(event) => { event.preventDefault(); navigate('/demo'); }}>Try it with sample data</a>
          <p>Try a ready client action room in one click. Nothing is saved to your account.</p>
        </div>
        {#if notice}<p class="inline-notice success" role="status">{notice}</p>{/if}
        <ul class="fact-lines" aria-label="Product facts">
          <li>Clients approve, choose, upload a PDF, or open an HTTPS link without an account.</li>
          <li>An expired client link cannot read or submit the request.</li>
          <li>Demo changes are sample-only and resettable.</li>
        </ul>
      </div>
      <div class="service-scene">
        <img src="/archive-window.svg" width="720" height="520" alt="Client requests pass through one focused service window into a dated record." fetchpriority="high" />
        <div class="preview-window" aria-label="Sample deadline queue">
          <p class="window-label">Alder Street Bakery launch</p>
          <ol>
            <li><span class="date-tab danger">27</span><span>Signed allergen sheet<small>Overdue</small></span></li>
            <li><span class="date-tab">28</span><span>Final menu proof<small>Due today</small></span></li>
            <li><span class="date-tab">29</span><span>Launch photo crop<small>Due tomorrow</small></span></li>
          </ol>
        </div>
      </div>
    </section>

    <section class="product-preview" aria-labelledby="preview-title">
      <div class="section-heading"><p class="eyebrow">The product</p><h2 id="preview-title">See the next client action first</h2></div>
      <div class="preview-ledger">
        <p class="ledger-number">01</p>
        <div><h3>Open requests are ordered by deadline</h3><p>Each link shows one action. Every answer, upload, choice, and outbound visit gets a server-timed audit entry.</p></div>
        <a class="text-link" href="/demo" onclick={(event) => { event.preventDefault(); navigate('/demo'); }}>Open the working action room</a>
      </div>
    </section>

    <section id="how-it-works" class="steps" aria-labelledby="steps-title">
      <div class="section-heading"><p class="eyebrow">How it works</p><h2 id="steps-title">Move one request through the window</h2></div>
      <ol>
        <li><span>1</span><div><h3>Issue the action</h3><p>Name the request and its deadline. The sample issues a seven-day client link.</p></div></li>
        <li><span>2</span><div><h3>Let the client act</h3><p>The client answers, chooses, uploads a checked PDF, or opens the named HTTPS site.</p></div></li>
        <li><span>3</span><div><h3>Read the record</h3><p>The audit shows the decision, actor name, and server time.</p></div></li>
      </ol>
    </section>

    <section class="boundary" aria-labelledby="boundary-title">
      <div class="section-heading"><p class="eyebrow">A narrow promise</p><h2 id="boundary-title">This is not a project board</h2></div>
      <div class="boundary-copy">
        <p>Client Action Room does not edit documents, run chat, or claim a payment succeeded. It records focused client actions.</p>
        <p>The sample uses a temporary server-side room. Reminder actions are recorded but no sample email is sent.</p>
        <a class="text-link" href="/privacy" onclick={(event) => { event.preventDefault(); navigate('/privacy'); }}>Read the privacy details</a>
      </div>
    </section>
  </main>

{:else if route === 'demo' || (route === 'workspace' && staffProfile)}
  <main id="main" class="app-page" tabindex="-1">
    <section class="page-intro">
      <p class="eyebrow">Northline Studio · {route === 'demo' ? 'sample workspace' : 'firm workspace'}</p>
      <h1 tabindex="-1">{route === 'demo' ? 'Your sample client action room' : 'Your firm action room'}</h1>
      <p>{route === 'demo' ? 'Open any scoped client action, complete it as Maya, then read the dated record.' : 'Create and issue client actions. Your workspace returns when you sign in again.'}</p>
    </section>
    {#if notice}<p class="inline-notice success" role="status">{notice}</p>{/if}
    {#if error}
      <div class="inline-notice danger" role="alert"><p>{error}</p>{#if requestId}<small>Request ID: {requestId}</small>{/if}<button class="text-button" type="button" onclick={() => loadDemo()}>Try again</button></div>
    {/if}
    {#if loading}
      <section class="skeleton" aria-busy="true" aria-label="Loading the sample room"><span></span><span></span><span></span></section>
    {:else if demo}
      <div class="workspace-header">
        <div><p class="meta-label">Client workspace</p><h2>{demo.workspace}</h2></div>
        <dl><div><dt>Owner</dt><dd>{demo.staff_owner}</dd></div><div><dt>Client</dt><dd>{demo.client_actor}</dd></div><div><dt>{route === 'demo' ? 'Demo ends' : 'Retention review'}</dt><dd>{formatDate(demo.expires_at)}</dd></div></dl>
      </div>
      <div class="staff-layout">
        <section class="deadline-rail" aria-labelledby="queue-title">
          <div class="rail-heading"><h2 id="queue-title">Requests by deadline</h2><span>{sortedActions.filter((action) => action.status === 'open').length} open</span></div>
          <ol>
            {#each sortedActions as action (action.id)}
              <li class="action-slip" data-due={action.due_at} data-kind={action.kind}>
                <div class="slip-top"><StatusStamp status={displayStatus(action, new Date(demo.server_now))} /><time datetime={action.due_at}>{dueLabel(action, demo.server_now)}</time></div>
                <h3>{action.title}</h3>
                <p>{action.instructions}</p>
                {#if action.status === 'completed'}
                  <p class="recorded-note">Maya’s answer is in the audit record.</p>
                {:else}
                  <div class="button-row">
                    <button class="button primary compact" type="button" disabled={busy} onclick={() => publish(action)}>{action.kind === 'approval' ? 'Publish client link' : `Open ${action.kind === 'external_link' ? 'external' : action.kind} request`}</button>
                    <button class="button secondary compact" type="button" disabled={busy} onclick={() => scheduleReminder(action)}>Schedule reminder</button>
                  </div>
                {/if}
              </li>
            {/each}
          </ol>
        </section>
        <aside class="detail-sheet" aria-labelledby="share-title">
          <p class="sheet-number">Service opening 01</p>
          <h2 id="share-title">Share one action</h2>
          <p>Each link can read and complete only its named action. It expires after seven days.</p>
          {#if publishedLink}
            <div class="share-result" role="status">
              <p><strong>Client link ready</strong></p>
              <code>{new URL(publishedLink.path, window.location.origin).toString()}</code>
              <div class="button-row">
                <a class="button primary" data-testid="open-client-link" href={publishedLink.path} target="_blank" rel="noopener">Open client request</a>
                <button class="button secondary" type="button" onclick={copyClientLink}>Copy link</button>
              </div>
            </div>
          {:else}
            <p class="empty-note">Open an action above to create its scoped client link.</p>
          {/if}
          <details class="composer">
            <summary>Create another approval</summary>
            <form onsubmit={createApproval} novalidate>
              {#if composerError}<p class="error-summary" role="alert">{composerError}</p>{/if}
              <label for="new-title">Approval name</label>
              <input id="new-title" maxlength="120" bind:value={newTitle} required />
              <label for="new-instructions">What should the client review?</label>
              <textarea id="new-instructions" rows="3" maxlength="2000" bind:value={newInstructions} required></textarea>
              <label for="new-deadline">Deadline</label>
              <input id="new-deadline" type="datetime-local" bind:value={newDeadline} required />
              <button class="button secondary" type="submit" disabled={busy}>Create approval</button>
            </form>
          </details>
          <div class="expiry-example">
            <h3>Check an expired link</h3>
            <p>This fixture proves expired links reveal no request content.</p>
            {#if expiredLink}
              <a class="text-link" data-testid="expired-client-link" href={expiredLink.path} target="_blank" rel="noopener">Open expired link example</a>
            {:else}
              <button class="text-button" type="button" disabled={busy} onclick={makeExpiredLink}>Create expired link example</button>
            {/if}
          </div>
        </aside>
      </div>
      <section class="audit-ledger" aria-labelledby="audit-title">
        <div class="rail-heading"><h2 id="audit-title">Audit record</h2><span>Server time</span></div>
        {#if demo.audit.length}
          <ol>
            {#each demo.audit as event (event.id)}
              <li data-event={event.event_name}>
                <span class="ledger-mark" aria-hidden="true"></span>
                <div><strong>{formatEventName(event)}</strong><p>{event.actor_label}{event.decision ? ` · ${event.decision === 'approved' ? 'Approved' : event.decision === 'changes_requested' ? 'Changes requested' : event.decision}` : ''}</p></div>
                <time datetime={event.occurred_at}>{formatDate(event.occurred_at)}</time>
              </li>
            {/each}
          </ol>
        {:else}
          <p>No events yet. Publish the approval to add the first link event.</p>
        {/if}
      </section>
    {/if}
  </main>

{:else if route === 'client'}
  <main id="main" class="client-page" tabindex="-1">
    {#if loading}
      <section class="client-window skeleton" aria-busy="true"><h1 tabindex="-1">Opening the client request</h1><span></span><span></span></section>
    {:else if clientExpired}
      <section class="client-window expired-window">
        <p class="eyebrow">Access ended</p>
        <h1 tabindex="-1">This client link has expired</h1>
        <p>{error || 'Ask Northline Studio for a new link.'}</p>
        <p>No request or workspace details were shown.</p>
        <a class="button secondary" href="/" onclick={(event) => { event.preventDefault(); navigate('/'); }}>Return home</a>
      </section>
    {:else if client}
      <section class="client-window">
        <div class="firm-line"><span class="firm-mark" aria-hidden="true"></span><div><strong>{client.firm}</strong><small>{client.workspace}</small></div></div>
        <p class="eyebrow">{client.action.kind === 'approval' ? 'Approval requested' : client.action.kind === 'upload' ? 'File requested' : client.action.kind === 'choice' ? 'Choice requested' : 'External action requested'}</p>
        <h1 tabindex="-1">{client.action.title}</h1>
        <p class="client-instructions">{client.action.instructions}</p>
        <dl class="scope-list"><div><dt>Deadline</dt><dd><time datetime={client.action.due_at}>{formatDate(client.action.due_at)}</time></dd></div><div><dt>This link can see</dt><dd>This {client.action.kind === 'external_link' ? 'external action' : client.action.kind} only</dd></div><div><dt>Link expires</dt><dd><time datetime={client.link_expires_at}>{formatDate(client.link_expires_at)}</time></dd></div></dl>
        {#if notice}<p class="inline-notice success" role="status">{notice}</p>{/if}
        {#if submission || completion}
          <div class="completion-record" data-testid="client-completion">
            <StatusStamp status="complete" />
            {#if submission}
              <h2>{submission.decision === 'approved' ? 'Approval recorded' : 'Changes requested'}</h2>
              <p>{submission.actor_label} · <time datetime={submission.occurred_at}>{formatDate(submission.occurred_at)}</time></p>
              {#if submission.comment}<p class="client-comment">“{submission.comment}”</p>{/if}
              <p class="legal-note">This is an action record, not a regulated electronic signature.</p>
            {:else if completion}
              <h2>{completion.kind === 'upload' ? 'File received and scanned' : completion.kind === 'choice' ? 'Choice recorded' : 'External link opened'}</h2>
              <p>{completion.actor_label} · <time datetime={completion.occurred_at}>{formatDate(completion.occurred_at)}</time></p>
              <p>{completion.detail}</p>
              {#if completion.destination_url}<a class="button primary" href={completion.destination_url} target="_blank" rel="noopener noreferrer">Continue to {client.destination_host}</a>{/if}
            {/if}
          </div>
        {:else if client.action.kind === 'approval'}
          <form class="approval-form" onsubmit={submitApproval} novalidate>
            <h2>Record your answer</h2>
            {#if formError}<div id="approval-error" class="error-summary" role="alert" tabindex="-1">{formError}</div>{/if}
            <label for="actor-label">Your name</label>
            <input id="actor-label" name="actor-label" maxlength="80" autocomplete="name" bind:value={actorLabel} required />
            <fieldset>
              <legend>Your answer</legend>
              <label class="radio-row"><input type="radio" name="decision" value="approved" bind:group={decision} /><span><strong>Approve this proof</strong><small>Record that the menu proof is approved.</small></span></label>
              <label class="radio-row"><input type="radio" name="decision" value="changes_requested" bind:group={decision} /><span><strong>Ask for a change</strong><small>Tell Northline Studio what needs attention.</small></span></label>
            </fieldset>
            <label for="comment">Note {decision === 'changes_requested' ? '(required)' : '(optional)'}</label>
            <textarea id="comment" name="comment" rows="4" maxlength="1000" bind:value={comment} aria-describedby="comment-help"></textarea>
            <small id="comment-help">The note becomes part of this sample audit record.</small>
            <button class="button primary" type="submit" disabled={busy}>{busy ? 'Recording answer…' : 'Record my answer'}</button>
            <p class="legal-note">This records your intent. It is not a regulated electronic signature.</p>
          </form>
        {:else if client.action.kind === 'choice'}
          <form class="approval-form" onsubmit={submitChoice} novalidate>
            <h2>Choose one crop</h2>
            {#if formError}<div class="error-summary" role="alert">{formError}</div>{/if}
            <label for="choice-actor">Your name</label>
            <input id="choice-actor" maxlength="80" autocomplete="name" bind:value={actorLabel} required />
            <fieldset><legend>Photo crop</legend>
              {#each client.choices as option}
                <label class="radio-row"><input type="radio" name="crop" value={option.key} bind:group={selectedChoice} /><span><strong>{option.label}</strong></span></label>
              {/each}
            </fieldset>
            <button class="button primary" type="submit" disabled={busy}>{busy ? 'Recording choice…' : 'Record my choice'}</button>
          </form>
        {:else if client.action.kind === 'upload'}
          <form class="approval-form" onsubmit={submitUpload} novalidate>
            <h2>Upload the signed sheet</h2>
            {#if formError}<div class="error-summary" role="alert">{formError}</div>{/if}
            <label for="upload-actor">Your name</label>
            <input id="upload-actor" maxlength="80" autocomplete="name" bind:value={actorLabel} required />
            <label for="client-file">Signed sheet (PDF, up to 5 MB)</label>
            <input id="client-file" type="file" accept="application/pdf,.pdf" required onchange={(event) => selectedFile = event.currentTarget.files?.[0] ?? null} />
            <p class="legal-note">The sample checks the real file type and known test malware before recording it. Demo files expire within 24 hours.</p>
            <button class="button primary" type="submit" disabled={busy}>{busy ? 'Scanning file…' : 'Upload and scan file'}</button>
          </form>
        {:else}
          <div class="approval-form">
            <h2>Open the hosted invoice</h2>
            {#if formError}<div class="error-summary" role="alert">{formError}</div>{/if}
            <label for="visit-actor">Your name</label>
            <input id="visit-actor" maxlength="80" autocomplete="name" bind:value={actorLabel} required />
            <p>The destination is <strong>{client.destination_host}</strong>. Client Action Room records that you opened it. It does not claim payment.</p>
            <button class="button primary" type="button" disabled={busy} onclick={openExternal}>{busy ? 'Checking link…' : `Open ${client.destination_host}`}</button>
          </div>
        {/if}
      </section>
    {:else}
      <section class="client-window"><h1 tabindex="-1">We could not open this request</h1><p>{error}</p><button class="button secondary" type="button" onclick={loadClient}>Try again</button></section>
    {/if}
  </main>

{:else if route === 'workspace' || route === 'auth-callback'}
  <main id="main" class="prose-page" tabindex="-1">
    <p class="eyebrow">Private staff area</p>
    <h1 tabindex="-1">Open your firm workspace</h1>
    {#if loading}
      <p role="status">Checking your Sociobot sign-in…</p>
    {:else if staffProfile}
      <p class="lede">Signed in as {staffProfile.name || staffProfile.email}. Your identity is keyed by its stable tenant ID.</p>
      <section><h2>Staff identity is ready</h2><p>Client links remain account-free and scoped to one action. Staff access uses Microsoft Entra External ID.</p></section>
      <button class="button secondary" type="button" onclick={leaveWorkspace}>Sign out</button>
    {:else}
      {#if error}<p class="inline-notice danger" role="alert">{error}</p>{/if}
      <p class="lede">Sign in with the shared Sociobot customer tenant. Client access never requires an account.</p>
      <button class="button primary" type="button" onclick={startForReal}>Sign in with Sociobot</button>
    {/if}
  </main>

{:else if route === 'privacy'}
  <main id="main" class="prose-page" tabindex="-1">
    <p class="eyebrow">Privacy</p>
    <h1 tabindex="-1">How Client Action Room handles data</h1>
    <p class="lede">The demo stores temporary sample changes and checked sample files. It does not create a firm account.</p>
    <section><h2>What the demo stores</h2><p>The server assigns a random demo session and keeps its sample actions, link digests, answers, and audit times for up to 24 hours. A browser cookie holds only the random session reference.</p></section>
    <section><h2>What client links reveal</h2><p>A client link can open one approval. The secret stays in the URL fragment, is exchanged once, and is not stored in server logs. The database stores a one-way digest.</p></section>
    <section><h2>What we do not collect</h2><p>The demo has no analytics, advertising, account profiles, payment collection, or email delivery. Outbound sites open only after you choose them.</p></section>
    <section><h2>Deletion and contact</h2><p>Resetting or leaving the demo deletes that sample room. Expired rooms are purged within an hour. For privacy questions, email <a href="mailto:privacy@sociobot.in">privacy@sociobot.in</a>.</p></section>
  </main>

{:else if route === 'terms'}
  <main id="main" class="prose-page" tabindex="-1">
    <p class="eyebrow">Terms</p>
    <h1 tabindex="-1">Terms for Client Action Room</h1>
    <p class="lede">These terms cover the public demo. Paid firm accounts are not offered in this release.</p>
    <section><h2>Use the demo lawfully</h2><p>Use the sample room to evaluate the approval flow. Do not enter confidential, regulated, illegal, or third-party personal information.</p></section>
    <section><h2>Approval records</h2><p>The demo records an approval or change request for evaluation. It is not a regulated electronic signature, legal advice, or proof of identity.</p></section>
    <section><h2>Availability and sample data</h2><p>Sample rooms expire after 24 hours and may be removed sooner for security or maintenance. Resetting or leaving removes the current sample.</p></section>
    <section><h2>Contact</h2><p>Questions about these terms can be sent to <a href="mailto:legal@sociobot.in">legal@sociobot.in</a>.</p></section>
  </main>

{:else}
  <main id="main" class="not-found-page" tabindex="-1">
    <div class="empty-window" aria-hidden="true"><span></span></div>
    <p class="eyebrow">404 · empty docket</p>
    <h1 tabindex="-1">This record is not in the archive</h1>
    <p>The address does not match a Client Action Room page.</p>
    <a class="button primary" href="/" onclick={(event) => { event.preventDefault(); navigate('/'); }}>Return to the front desk</a>
  </main>
{/if}

<SiteFooter {navigate} />
