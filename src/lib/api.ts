export class ApiError extends Error {
  constructor(
    public readonly status: number,
    public readonly code: string,
    message: string,
    public readonly requestId?: string,
  ) {
    super(message);
  }
}

export async function api<T>(path: string, init?: RequestInit): Promise<T> {
  let response: Response;
  try {
    response = await fetch(path, {
      credentials: 'same-origin',
      ...init,
      headers: {
        Accept: 'application/json',
        ...(init?.body ? { 'Content-Type': 'application/json' } : {}),
        ...init?.headers,
      },
    });
  } catch {
    throw new ApiError(0, 'offline', 'You are offline. Reconnect and try again.');
  }

  if (!response.ok) {
    const body = (await response.json().catch(() => null)) as
      | { code?: string; message?: string; request_id?: string }
      | null;
    throw new ApiError(
      response.status,
      body?.code ?? 'request_failed',
      body?.message ?? 'We could not finish that request. Try again.',
      body?.request_id,
    );
  }
  if (response.status === 204) return undefined as T;
  return (await response.json()) as T;
}
