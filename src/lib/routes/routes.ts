export interface RouteMeta {
  title: string;
  description: string;
  canonicalPath: string;
}

export type RouteName = 'home' | 'demo' | 'client' | 'privacy' | 'terms' | 'not-found';

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
  if (pathname === '/privacy') return 'privacy';
  if (pathname === '/terms') return 'terms';
  return 'not-found';
}
