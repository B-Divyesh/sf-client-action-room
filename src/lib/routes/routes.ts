export interface RouteMeta {
  title: string;
  description: string;
  canonicalPath: string;
}

export type RouteName = 'home' | 'demo' | 'client' | 'workspace' | 'auth-callback' | 'privacy' | 'terms' | 'not-found';

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
  if (pathname === '/privacy') return 'privacy';
  if (pathname === '/terms') return 'terms';
  return 'not-found';
}
