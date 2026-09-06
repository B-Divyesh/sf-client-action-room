export interface RouteMeta {
  title: string;
  description: string;
  canonicalPath: string;
}

export type RouteName =
  | 'home'
  | 'demo'
  | 'client'
  | 'workspace'
  | 'auth-callback'
  | 'onboarding'
  | 'app'
  | 'new-action'
  | 'settings'
  | 'billing'
  | 'privacy'
  | 'terms'
  | 'not-found';

export const routeMeta: Record<RouteName, RouteMeta> = {
  home: {
    title: 'Client Action Room — get client actions done',
    description: 'Give each client one short action list for approvals, files, choices, and links.',
    canonicalPath: '/',
  },
  demo: {
    title: 'Demo — Client Action Room',
    description: 'Try an isolated sample client action room and complete one approval.',
    canonicalPath: '/demo',
  },
  client: {
    title: 'Client request — Client Action Room',
    description: 'Review one request and record your answer without creating an account.',
    canonicalPath: '/client',
  },
  workspace: {
    title: 'Workspace — Client Action Room',
    description: 'Open your signed-in Client Action Room workspace.',
    canonicalPath: '/workspace',
  },
  'auth-callback': {
    title: 'Signing in — Client Action Room',
    description: 'Finish signing in to Client Action Room.',
    canonicalPath: '/auth/callback',
  },
  onboarding: {
    title: 'Set up your firm — Client Action Room',
    description: 'Name your firm and first client workspace.',
    canonicalPath: '/onboarding',
  },
  app: {
    title: 'Action queue — Client Action Room',
    description: 'Create, share, and review client approval actions.',
    canonicalPath: '/app',
  },
  'new-action': {
    title: 'New action — Client Action Room',
    description: 'Create one approval request and set its deadline.',
    canonicalPath: '/app/workspaces/new/actions/new',
  },
  settings: {
    title: 'Settings — Client Action Room',
    description: 'Manage firm retention, staff access, export, and deletion.',
    canonicalPath: '/app/settings',
  },
  billing: {
    title: 'Plans — Client Action Room',
    description: 'Read the current recurring checkout status for your firm.',
    canonicalPath: '/app/billing',
  },
  privacy: {
    title: 'Privacy — Client Action Room',
    description: 'Read how Client Action Room handles sample, client, and account data.',
    canonicalPath: '/privacy',
  },
  terms: {
    title: 'Terms — Client Action Room',
    description: 'Read the terms for using Client Action Room and its approval records.',
    canonicalPath: '/terms',
  },
  'not-found': {
    title: 'Page not found — Client Action Room',
    description: 'This Client Action Room page could not be found.',
    canonicalPath: '/404',
  },
};

export function resolveRoute(pathname: string, search = ''): RouteName {
  if (pathname === '/' && new URLSearchParams(search).get('demo') === '1') return 'demo';
  if (pathname === '/') return 'home';
  if (pathname === '/demo') return 'demo';
  if (pathname === '/client') return 'client';
  if (pathname === '/workspace') return 'workspace';
  if (pathname === '/auth/callback') return 'auth-callback';
  if (pathname === '/onboarding') return 'onboarding';
  if (pathname === '/app') return 'app';
  if (/^\/app\/workspaces\/[^/]+\/actions\/new$/.test(pathname)) return 'new-action';
  if (pathname === '/app/settings') return 'settings';
  if (pathname === '/app/billing') return 'billing';
  if (pathname === '/privacy') return 'privacy';
  if (pathname === '/terms') return 'terms';
  return 'not-found';
}
