<script lang="ts">
  import { onMount, tick } from 'svelte';
  import ArchiveHeader from './lib/components/ArchiveHeader.svelte';
  import DemoBanner from './lib/components/DemoBanner.svelte';
  import SiteFooter from './lib/components/SiteFooter.svelte';
  import StatusStamp from './lib/components/StatusStamp.svelte';
  import { api, ApiError } from './lib/api';
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
  type OrganizationOverview = {
    id: string;
    name: string;
    time_zone: string;
    region: string;
    retention_days: number;
    deletion_due_at: string | null;
    role: string;
    workspaces: Array<{ id: string; client_label: string; client_actor: string; open_actions: number }>;
    members: Array<{ oid: string; display_name: string; role: string }>;
    subscription: { tier: string; status: string; period_end: string | null; verified_at: string } | null;
    recurring_billing_available: boolean;
  };
  let staffProfile: { id: string; name: string; email: string; has_workspace: boolean; organization_id: string | null; role: string | null } | null = null;
  let organization: OrganizationOverview | null = null;
  let staffAccessToken = '';
  let firmName = '';
  let workspaceName = '';
  let clientName = '';
  let onboardingError = '';
  let settingsName = '';
  let settingsTimeZone = 'America/New_York';
  let settingsRetention = 90;
  let deletionConfirmation = '';
  let inviteRole = 'member';
  let invitationPath = '';
  let billingStatus: { available: boolean; reason: string; subscription: OrganizationOverview['subscription']; export_available: boolean } | null = null;

  $: meta = routeMeta[route];
  $: canonical = `${canonicalOrigin}${route === 'new-action' ? pathname : meta.canonicalPath}`;
  $: sortedActions = demo ? orderedByDeadline(demo.actions) : [];
  $: activeWorkspaceId = route === 'new-action' ? pathname.split('/')[3] ?? '' : organization?.workspaces[0]?.id ?? '';

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
    if (['workspace', 'auth-callback', 'onboarding', 'app', 'new-action', 'settings', 'billing'].includes(route)) await loadWorkspace();
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
      const { beginStaffSignIn } = await import('./lib/auth');
      await beginStaffSignIn();
    } catch (caught) {
      busy = false;
      showError(caught);
      navigate('/workspace');
    }
  }

  async function loadWorkspace() {
    loading = true;
    demo = null;
    try {
      const { finishStaffSignIn, staffToken } = await import('./lib/auth');
      staffAccount = await finishStaffSignIn();
      if (route === 'auth-callback') {
        navigate('/app');
        return;
      }
      if (staffAccount) {
        const token = await staffToken(staffAccount);
        staffAccessToken = token;
        staffProfile = await api<{ id: string; name: string; email: string; has_workspace: boolean; organization_id: string | null; role: string | null }>('/api/v1/me', {
          headers: { Authorization: `Bearer ${token}` },
        });
        const inviteToken = route === 'settings' ? new URLSearchParams(window.location.hash.slice(1)).get('invite') : null;
        if (inviteToken && !staffProfile.organization_id) {
          organization = await api<OrganizationOverview>('/api/v1/staff/members/invitations/accept', {
            method: 'POST',
            headers: { Authorization: `Bearer ${token}` },
            body: JSON.stringify({ token: inviteToken }),
          });
          window.history.replaceState({}, '', '/app/settings');
          staffProfile = { ...staffProfile, organization_id: organization.id, role: organization.role, has_workspace: organization.workspaces.length > 0 };
          notice = 'Staff invitation accepted.';
        }
        if (staffProfile.organization_id) {
          organization = await api<OrganizationOverview>('/api/v1/staff/organization', {
            headers: { Authorization: `Bearer ${token}` },
          });
          settingsName = organization.name;
          settingsTimeZone = organization.time_zone;
          settingsRetention = organization.retention_days;
        }
        if (route === 'billing' && staffProfile.organization_id) {
          billingStatus = await api('/api/v1/billing/entitlement', {
            headers: { Authorization: `Bearer ${token}` },
          });
        }
        if (staffProfile.has_workspace && route !== 'settings' && route !== 'billing') {
          const queuePath = route === 'new-action'
            ? `/api/v1/staff/workspaces/${encodeURIComponent(activeWorkspaceId)}`
            : '/api/v1/staff/workspace';
          demo = await api<DemoQueue>(queuePath, {
            headers: { Authorization: `Bearer ${token}` },
          });
        } else {
          demo = null;
        }
      }
    } catch (caught) { showError(caught); }
    finally { loading = false; }
  }

  async function createWorkspace(event: SubmitEvent) {
    event.preventDefault();
    onboardingError = '';
    if (!firmName.trim() || !workspaceName.trim() || !clientName.trim()) {
      onboardingError = 'Name your firm, client workspace, and client.';
      return;
    }
    busy = true;
    try {
      demo = await api<DemoQueue>('/api/v1/staff/workspace', {
        method: 'POST',
        headers: { Authorization: `Bearer ${staffAccessToken}` },
        body: JSON.stringify({
          firm_name: firmName,
          client_label: workspaceName,
          client_actor: clientName,
        }),
      });
      if (staffProfile) staffProfile = { ...staffProfile, has_workspace: true };
      organization = await api<OrganizationOverview>('/api/v1/staff/organization', {
        headers: { Authorization: `Bearer ${staffAccessToken}` },
      });
      notice = 'Your firm workspace is ready. Create the first approval.';
      if (route === 'onboarding') navigate('/app');
    } catch (caught) {
      onboardingError = caught instanceof Error ? caught.message : 'We could not create the workspace. Try again.';
    } finally {
      busy = false;
    }
  }

  async function saveSettings(event: SubmitEvent) {
    event.preventDefault();
    busy = true;
    error = '';
    try {
      organization = await api<OrganizationOverview>('/api/v1/staff/organization', {
        method: 'PATCH',
        headers: { Authorization: `Bearer ${staffAccessToken}` },
        body: JSON.stringify({ name: settingsName, time_zone: settingsTimeZone, retention_days: settingsRetention }),
      });
      notice = 'Firm settings saved.';
    } catch (caught) { showError(caught); }
    finally { busy = false; }
  }

  async function downloadExport() {
    busy = true;
    error = '';
    try {
      const response = await fetch('/api/v1/staff/organization/export', {
        credentials: 'same-origin',
        headers: { Authorization: `Bearer ${staffAccessToken}` },
      });
      if (!response.ok) throw new Error('export failed');
      const url = URL.createObjectURL(await response.blob());
      const link = document.createElement('a');
      link.href = url;
      link.download = 'client-action-room-export.json';
      link.click();
      URL.revokeObjectURL(url);
      notice = 'Your firm export downloaded.';
    } catch { error = 'We could not prepare the export. Try again.'; }
    finally { busy = false; }
  }

  async function scheduleDeletion(event: SubmitEvent) {
    event.preventDefault();
    busy = true;
    try {
      const result = await api<{ deletion_due_at: string }>('/api/v1/staff/organization', {
        method: 'DELETE',
        headers: { Authorization: `Bearer ${staffAccessToken}` },
        body: JSON.stringify({ confirmation: deletionConfirmation }),
      });
      if (organization) organization = { ...organization, deletion_due_at: result.deletion_due_at };
      notice = 'Firm deletion is scheduled. You can cancel it before the date shown.';
    } catch (caught) { showError(caught); }
    finally { busy = false; }
  }

  async function cancelDeletion() {
    busy = true;
    try {
      await api('/api/v1/staff/organization/deletion/cancel', {
        method: 'POST', headers: { Authorization: `Bearer ${staffAccessToken}` },
      });
      if (organization) organization = { ...organization, deletion_due_at: null };
      deletionConfirmation = '';
      notice = 'Firm deletion cancelled.';
    } catch (caught) { showError(caught); }
    finally { busy = false; }
  }

  async function createInvitation() {
    busy = true;
    try {
      const result = await api<{ path: string }>('/api/v1/staff/members/invitations', {
        method: 'POST',
        headers: { Authorization: `Bearer ${staffAccessToken}` },
        body: JSON.stringify({ role: inviteRole }),
      });
      invitationPath = new URL(result.path, window.location.origin).toString();
      notice = 'Staff invitation link ready.';
    } catch (caught) { showError(caught); }
    finally { busy = false; }
  }

  async function leaveWorkspace() {
    if (staffAccount) {
      const { signOut } = await import('./lib/auth');
      await signOut(staffAccount);
    }
  }

  async function publish(action: DemoAction) {
    busy = true;
    error = '';
    try {
      const root = route === 'demo' ? '/api/v1/demo' : '/api/v1/staff';
      publishedLink = await api<{ path: string; expires_at: string }>(
        `${root}/actions/${encodeURIComponent(action.id)}/publish`,
        { method: 'POST', headers: { 'Idempotency-Key': crypto.randomUUID(), ...(route !== 'demo' ? { Authorization: `Bearer ${staffAccessToken}` } : {}) } },
      );
      demo = await api<DemoQueue>(route === 'demo' ? '/api/v1/demo/queue' : '/api/v1/staff/workspace', {
        headers: route !== 'demo' ? { Authorization: `Bearer ${staffAccessToken}` } : {},
      });
      notice = 'The client link is ready. It can open only this action.';
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
      const actionPath = route === 'demo'
        ? '/api/v1/demo/actions'
        : route === 'new-action'
          ? `/api/v1/staff/workspaces/${encodeURIComponent(activeWorkspaceId)}/actions`
          : '/api/v1/staff/actions';
      await api<DemoAction>(actionPath, {
        method: 'POST',
        headers: { 'Idempotency-Key': crypto.randomUUID(), ...(route !== 'demo' ? { Authorization: `Bearer ${staffAccessToken}` } : {}) },
        body: JSON.stringify({
          title: newTitle,
          instructions: newInstructions,
          due_at: new Date(newDeadline).toISOString(),
        }),
      });
      demo = await api<DemoQueue>(route === 'demo' ? '/api/v1/demo/queue' : '/api/v1/staff/workspace', {
        headers: route !== 'demo' ? { Authorization: `Bearer ${staffAccessToken}` } : {},
      });
      newTitle = '';
      newInstructions = '';
      newDeadline = '';
      notice = 'The new approval is in the deadline list.';
      if (route === 'new-action') navigate('/app');
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
    if (!client || !selectedFile) { formError = 'Choose one PDF file no larger than 5 MB.'; return; }
    const form = new FormData();
    form.set('actor_label', actorLabel);
    form.set('file', selectedFile);
    busy = true; formError = '';
    try {
      completion = await api<Completion>(`/api/v1/client/actions/${encodeURIComponent(client.action.id)}/upload`, { method: 'POST', body: form });
      notice = 'Your file passed the malware scan and is recorded.';
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
      const root = route === 'demo' ? '/api/v1/demo' : '/api/v1/staff';
      const result = await api<{ scheduled_for: string }>(`${root}/actions/${encodeURIComponent(action.id)}/reminder`, {
        method: 'POST',
        headers: route !== 'demo' ? { Authorization: `Bearer ${staffAccessToken}` } : {},
      });
      demo = await api<DemoQueue>(route === 'demo' ? '/api/v1/demo/queue' : '/api/v1/staff/workspace', {
        headers: route !== 'demo' ? { Authorization: `Bearer ${staffAccessToken}` } : {},
      });
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
{#if route === 'demo' || (route === 'client' && client?.namespace === 'demo')}
  <DemoBanner busy={busy} onReset={() => navigate('/demo?reset=1')} onStart={startForReal} />
{/if}
<ArchiveHeader route={route} {navigate} {theme} {toggleTheme} signedIn={Boolean(staffAccount)} signOut={leaveWorkspace} />

{#if route === 'home'}
  <main id="main" class="landing" tabindex="-1">
    <section class="hero" aria-labelledby="home-title">
      <div class="hero-copy">
        <p class="eyebrow">Client action requests</p>
        <h1 id="home-title" tabindex="-1">Get client actions done on time</h1>
        <p class="lede">For small firms chasing approvals, files, choices, and payment links across email.</p>
        <div class="primary-row">
          <a class="button primary" href="/demo" onclick={(event) => { event.preventDefault(); navigate('/demo'); }}>Try it with sample data</a>
          <p>Try a ready client action room in one click. Nothing is saved to your account.</p>
        </div>
        {#if notice}<p class="inline-notice success" role="status">{notice}</p>{/if}
        <ul class="fact-lines" aria-label="Product facts">
          <li>Clients approve, choose, upload a PDF, or open an HTTPS link without an account.</li>
          <li>Client links last seven days, then cannot read or submit the request.</li>
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
        <div><h3>Open requests are ordered by deadline</h3><p>Each link shows one action. Each completed action adds a server-timed audit entry.</p></div>
        <a class="text-link" href="/demo" onclick={(event) => { event.preventDefault(); navigate('/demo'); }}>Open the working action room</a>
      </div>
    </section>

    <section id="how-it-works" class="steps" aria-labelledby="steps-title">
      <div class="section-heading"><p class="eyebrow">How it works</p><h2 id="steps-title">Send, complete, and record a request</h2></div>
      <ol>
        <li><span>1</span><div><h3>Issue the action</h3><p>Name the request and its deadline. The sample issues a seven-day client link.</p></div></li>
        <li><span>2</span><div><h3>Let the client act</h3><p>The client answers, chooses, uploads a checked PDF, or opens the named HTTPS site.</p></div></li>
        <li><span>3</span><div><h3>Read the record</h3><p>The audit shows the decision, actor name, and server time.</p></div></li>
      </ol>
    </section>

    <section class="boundary" aria-labelledby="boundary-title">
      <div class="section-heading"><p class="eyebrow">Product limits</p><h2 id="boundary-title">This is not a project board</h2></div>
      <div class="boundary-copy">
        <p>Client Action Room does not edit documents, run chat, or claim a payment succeeded. It records focused client actions.</p>
        <p>The demo stays on this site. It records reminder schedules but does not send email.</p>
        <a class="text-link" href="/privacy" onclick={(event) => { event.preventDefault(); navigate('/privacy'); }}>Read the privacy details</a>
      </div>
    </section>

    <section class="pricing" aria-labelledby="pricing-title">
      <div class="section-heading"><p class="eyebrow">Pricing</p><h2 id="pricing-title">Planned recurring plans</h2></div>
      <div class="pricing-list">
        <p><strong>Starter · $49/month</strong><span>Five client workspaces and three staff seats.</span></p>
        <p><strong>Studio · $99/month</strong><span>Twenty client workspaces and ten staff seats.</span></p>
      </div>
      <p>Checkout is not available until Sociobot registers these recurring offers.</p>
      <a class="text-link" href="/app/billing" onclick={(event) => { event.preventDefault(); navigate('/app/billing'); }}>Check plan status</a>
    </section>
  </main>

{:else if route === 'onboarding' && staffProfile}
  <main id="main" class="prose-page" tabindex="-1">
    <p class="eyebrow">Firm setup</p>
    <h1 tabindex="-1">Set up your firm</h1>
    {#if staffProfile.has_workspace}
      <p class="lede">Your firm is ready. Open the queue to create or share an approval.</p>
      <a class="button primary" href="/app" onclick={(event) => { event.preventDefault(); navigate('/app'); }}>Open action queue</a>
    {:else}
      <form class="onboarding-form" onsubmit={createWorkspace} novalidate>
        <h2>Name your first client workspace</h2>
        <p>This starts empty. Sample data never moves into your firm account.</p>
        {#if onboardingError}<p class="error-summary" role="alert">{onboardingError}</p>{/if}
        <label for="firm-name">Firm name</label>
        <input id="firm-name" maxlength="80" bind:value={firmName} required />
        <label for="workspace-name">Client workspace name</label>
        <input id="workspace-name" maxlength="80" bind:value={workspaceName} required />
        <label for="client-name">Client name</label>
        <input id="client-name" maxlength="80" bind:value={clientName} required />
        <button class="button primary" type="submit" disabled={busy}>{busy ? 'Creating workspace…' : 'Create firm workspace'}</button>
      </form>
    {/if}
  </main>

{:else if route === 'new-action' && staffProfile}
  <main id="main" class="prose-page" tabindex="-1">
    <p class="eyebrow">Approval request</p>
    <h1 tabindex="-1">Create a client action</h1>
    <p class="lede">Name one approval, explain what to review, and set its deadline.</p>
    {#if error}<p class="inline-notice danger" role="alert">{error}</p>{/if}
    {#if demo}<form class="onboarding-form" onsubmit={createApproval} novalidate>
      {#if composerError}<p class="error-summary" role="alert">{composerError}</p>{/if}
      <label for="new-action-title">Approval name</label>
      <input id="new-action-title" maxlength="120" bind:value={newTitle} required />
      <label for="new-action-instructions">What should the client review?</label>
      <textarea id="new-action-instructions" rows="4" maxlength="2000" bind:value={newInstructions} required></textarea>
      <label for="new-action-deadline">Deadline</label>
      <input id="new-action-deadline" type="datetime-local" bind:value={newDeadline} required />
      <div class="button-row"><button class="button primary" type="submit" disabled={busy}>{busy ? 'Creating approval…' : 'Create approval'}</button><a class="button secondary" href="/app" onclick={(event) => { event.preventDefault(); navigate('/app'); }}>Cancel</a></div>
    </form>{:else if !loading}<a class="button secondary" href="/app" onclick={(event) => { event.preventDefault(); navigate('/app'); }}>Return to the action queue</a>{/if}
  </main>

{:else if route === 'settings' && staffProfile}
  <main id="main" class="prose-page" tabindex="-1">
    <p class="eyebrow">Firm controls</p>
    <h1 tabindex="-1">Manage firm settings</h1>
    {#if notice}<p class="inline-notice success" role="status">{notice}</p>{/if}
    {#if error}<p class="inline-notice danger" role="alert">{error}</p>{/if}
    {#if organization}
      <form class="settings-panel" onsubmit={saveSettings} novalidate>
        <h2>Firm record</h2>
        <label for="settings-name">Firm name</label>
        <input id="settings-name" maxlength="80" bind:value={settingsName} required disabled={organization.role === 'member'} />
        <label for="settings-zone">Time zone</label>
        <select id="settings-zone" bind:value={settingsTimeZone} disabled={organization.role === 'member'}>
          <option value="America/New_York">America/New_York</option>
          <option value="America/Chicago">America/Chicago</option>
          <option value="America/Los_Angeles">America/Los_Angeles</option>
          <option value="Europe/London">Europe/London</option>
          <option value="Asia/Kolkata">Asia/Kolkata</option>
          <option value="Etc/UTC">UTC</option>
        </select>
        <label for="settings-retention">Record retention</label>
        <select id="settings-retention" bind:value={settingsRetention} disabled={organization.role === 'member'}>
          <option value={30}>30 days after completion</option>
          <option value={90}>90 days after completion</option>
          <option value={365}>365 days after completion</option>
          <option value={0}>Until firm deletion</option>
        </select>
        <p class="field-note">Region moves are not available here. Export your records before requesting a move.</p>
        {#if organization.role !== 'member'}<button class="button primary" type="submit" disabled={busy}>Save firm settings</button>{:else}<p class="field-note">Ask a firm owner or admin to change these settings.</p>{/if}
      </form>
      <section class="settings-panel" aria-labelledby="people-title">
        <h2 id="people-title">People</h2>
        <ul class="member-list">{#each organization.members as member}<li><span>{member.display_name || 'Staff member'}</span><strong>{member.role}</strong></li>{/each}</ul>
        {#if organization.role !== 'member'}
          <label for="invite-role">New member role</label>
          <select id="invite-role" bind:value={inviteRole}><option value="member">Member</option><option value="admin">Admin</option></select>
          <button class="button secondary" type="button" disabled={busy} onclick={createInvitation}>Create staff invitation</button>
          {#if invitationPath}<p class="share-result"><strong>Invitation link</strong><code>{invitationPath}</code></p>{/if}
          <p class="field-note">Additional seats require an active recurring plan.</p>
        {:else}<p class="field-note">Ask a firm owner or admin to invite staff.</p>{/if}
      </section>
      <section class="settings-panel" aria-labelledby="data-title">
        <h2 id="data-title">Export and deletion</h2>
        {#if organization.role === 'owner'}
          <p>Download your firm record as JSON.</p>
          <button class="button secondary" type="button" disabled={busy} onclick={downloadExport}>Download firm export</button>
        {#if organization.deletion_due_at}
          <p class="inline-notice danger">Deletion is scheduled for <time datetime={organization.deletion_due_at}>{formatDate(organization.deletion_due_at)}</time>.</p>
          <button class="button secondary" type="button" disabled={busy} onclick={cancelDeletion}>Cancel firm deletion</button>
        {:else}
          <form class="danger-zone" onsubmit={scheduleDeletion} novalidate>
            <label for="deletion-confirmation">Type “{organization.name}” to schedule deletion</label>
            <input id="deletion-confirmation" bind:value={deletionConfirmation} required />
            <button class="button danger" type="submit" disabled={busy}>Schedule firm deletion</button>
            <p class="field-note">Deletion waits seven days. You can cancel during that time.</p>
          </form>
        {/if}
        {:else}<p>Only the firm owner can export or schedule deletion.</p>{/if}
      </section>
    {:else if !loading}
      <p>Create your firm before changing settings.</p>
      <a class="button primary" href="/onboarding" onclick={(event) => { event.preventDefault(); navigate('/onboarding'); }}>Set up your firm</a>
    {/if}
  </main>

{:else if route === 'billing' && staffProfile}
  <main id="main" class="prose-page" tabindex="-1">
    <p class="eyebrow">Recurring plan</p>
    <h1 tabindex="-1">Manage your plan</h1>
    {#if billingStatus?.subscription}
      <section class="settings-panel"><h2>Current subscription</h2><p><strong>{billingStatus.subscription.tier}</strong> · {billingStatus.subscription.status}</p>{#if billingStatus.subscription.period_end}<p>Current paid period ends <time datetime={billingStatus.subscription.period_end}>{formatDate(billingStatus.subscription.period_end)}</time>.</p>{/if}<p>Firm export stays available when the paid period ends.</p></section>
    {:else}
      <section class="settings-panel"><h2>Checkout is not available yet</h2><p>The recurring Sociobot offers still need factory registration. No payment details are collected here.</p><p>Your existing approval workspace and export remain available.</p></section>
    {/if}
  </main>

{:else if route === 'demo' || ((route === 'workspace' || route === 'app') && staffProfile)}
  <main id="main" class="app-page" tabindex="-1">
    <section class="page-intro">
      <p class="eyebrow">{demo?.firm ?? 'Firm workspace'} · {route === 'demo' ? 'sample workspace' : 'firm workspace'}</p>
      <h1 tabindex="-1">{route === 'demo' ? 'Your sample client action room' : 'Client action queue'}</h1>
      <p>{route === 'demo' ? 'Open any scoped client action, complete it as Maya, then read the dated record.' : 'Create and issue client actions. Your workspace returns when you sign in again.'}</p>
    </section>
    {#if notice}<p class="inline-notice success" role="status">{notice}</p>{/if}
    {#if error}
      <div class="inline-notice danger" role="alert"><p>{error}</p>{#if requestId}<small>Request ID: {requestId}</small>{/if}<button class="text-button" type="button" onclick={() => route === 'demo' ? loadDemo() : loadWorkspace()}>Try again</button></div>
    {/if}
    {#if loading}
      <section class="skeleton" aria-busy="true" aria-label="Loading the sample room"><span></span><span></span><span></span></section>
    {:else if route !== 'demo' && staffProfile && !demo}
      <form class="onboarding-form" onsubmit={createWorkspace} novalidate>
        <h2>Name your first client workspace</h2>
        <p>This starts empty. Sample data never moves into your firm account.</p>
        {#if onboardingError}<p class="error-summary" role="alert">{onboardingError}</p>{/if}
        <label for="firm-name">Firm name</label>
        <input id="firm-name" maxlength="80" bind:value={firmName} required />
        <label for="workspace-name">Client workspace name</label>
        <input id="workspace-name" maxlength="80" bind:value={workspaceName} required />
        <label for="client-name">Client name</label>
        <input id="client-name" maxlength="80" bind:value={clientName} required />
        <button class="button primary" type="submit" disabled={busy}>{busy ? 'Creating workspace…' : 'Create firm workspace'}</button>
      </form>
    {:else if demo}
      <div class="workspace-header">
        <div><p class="meta-label">Client workspace</p><h2>{demo.workspace}</h2></div>
        <dl><div><dt>Owner</dt><dd>{demo.staff_owner}</dd></div><div><dt>Client</dt><dd>{demo.client_actor}</dd></div>{#if route === 'demo'}<div><dt>Demo ends</dt><dd>{formatDate(demo.expires_at)}</dd></div>{/if}</dl>
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
                  <p class="recorded-note">{demo.client_actor}’s answer is in the audit record.</p>
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
          <p class="sheet-number">Client link</p>
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
          {#if route !== 'demo'}<a class="text-link" href={`/app/workspaces/${organization?.workspaces[0]?.id ?? 'new'}/actions/new`} onclick={(event) => { event.preventDefault(); navigate(`/app/workspaces/${organization?.workspaces[0]?.id ?? 'new'}/actions/new`); }}>Open the full action form</a>{/if}
          {#if route === 'demo'}<div class="expiry-example">
            <h3>Check an expired link</h3>
            <p>This fixture proves expired links reveal no request content.</p>
            {#if expiredLink}
              <a class="text-link" data-testid="expired-client-link" href={expiredLink.path} target="_blank" rel="noopener">Open expired link example</a>
            {:else}
              <button class="text-button" type="button" disabled={busy} onclick={makeExpiredLink}>Create expired link example</button>
            {/if}
          </div>{/if}
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
              <h2>{completion.kind === 'upload' ? 'File received and malware-scanned' : completion.kind === 'choice' ? 'Choice recorded' : 'External link opened'}</h2>
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
              <label class="radio-row"><input type="radio" name="decision" value="approved" bind:group={decision} /><span><strong>Approve this request</strong><small>Record that this request is approved.</small></span></label>
              <label class="radio-row"><input type="radio" name="decision" value="changes_requested" bind:group={decision} /><span><strong>Ask for a change</strong><small>Tell {client.firm} what needs attention.</small></span></label>
            </fieldset>
            <label for="comment">Note {decision === 'changes_requested' ? '(required)' : '(optional)'}</label>
            <textarea id="comment" name="comment" rows="4" maxlength="1000" bind:value={comment} aria-describedby="comment-help"></textarea>
            <small id="comment-help">The note becomes part of this audit record.</small>
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
            <p class="legal-note">The server checks the file type and scans it for malware before recording it. Files stay available for 24 hours.</p>
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

{:else if ['workspace', 'auth-callback', 'onboarding', 'app', 'new-action', 'settings', 'billing'].includes(route)}
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
    <section><h2>What the demo stores</h2><p>The server keeps sample actions, answers, files, and audit times for up to 24 hours.</p></section>
    <section><h2>What client links reveal</h2><p>A client link opens one action. The browser removes its secret from the address after exchange.</p></section>
    <section><h2>What the demo sends</h2><p>Demo traffic stays on this site. It does not send email, collect payment, or load advertising.</p></section>
    <section><h2>What a firm workspace stores</h2><p>A firm workspace stores its names, approval requests, client answers, membership roles, and audit times in this product’s SQLite database.</p></section>
    <section><h2>Export and deletion</h2><p>Firm owners can download a JSON record, choose retention, and cancel a scheduled deletion during its seven-day recovery window.</p></section>
    <section><h2>Contact</h2><p>Resetting or leaving deletes the current sample room. For privacy questions, email <a href="mailto:privacy@sociobot.in">privacy@sociobot.in</a>.</p></section>
  </main>

{:else if route === 'terms'}
  <main id="main" class="prose-page" tabindex="-1">
    <p class="eyebrow">Terms</p>
    <h1 tabindex="-1">Terms for Client Action Room</h1>
    <p class="lede">These terms cover the public demo and the current firm workspace. Recurring checkout is not available yet.</p>
    <section><h2>Use the demo lawfully</h2><p>Use the sample room to evaluate the approval flow. Do not enter confidential, regulated, illegal, or third-party personal information.</p></section>
    <section><h2>Approval records</h2><p>The demo records an approval or change request for evaluation. It is not a regulated electronic signature, legal advice, or proof of identity.</p></section>
    <section><h2>Availability and sample data</h2><p>Sample rooms expire after 24 hours and may be removed sooner for security or maintenance. Resetting or leaving removes the current sample.</p></section>
    <section><h2>Contact</h2><p>Questions about these terms can be sent to <a href="mailto:legal@sociobot.in">legal@sociobot.in</a>.</p></section>
  </main>

{:else}
  <main id="main" class="not-found-page" tabindex="-1">
    <div class="empty-window" aria-hidden="true"><span></span></div>
    <p class="eyebrow">404</p>
    <h1 tabindex="-1">We could not find this page</h1>
    <p>The address does not match a Client Action Room page.</p>
    <a class="button primary" href="/" onclick={(event) => { event.preventDefault(); navigate('/'); }}>Return home</a>
  </main>
{/if}

<SiteFooter {navigate} />
