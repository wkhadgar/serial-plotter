function cloneState<T>(state: T): T {
  if (typeof structuredClone === 'function') {
    return structuredClone(state);
  }

  return JSON.parse(JSON.stringify(state)) as T;
}

function canUseStorage(): boolean {
  return typeof window !== 'undefined' && typeof localStorage !== 'undefined';
}

export function loadWorkspaceState<T>(
  storageKey: string,
  fallback: T,
  revive: (parsed: unknown, fallback: T) => T
): T {
  const fallbackState = cloneState(fallback);

  if (!canUseStorage()) {
    return fallbackState;
  }

  try {
    const raw = localStorage.getItem(storageKey);
    if (!raw) {
      return fallbackState;
    }

    return revive(JSON.parse(raw), fallbackState);
  } catch (error) {
    console.error(`Erro ao carregar estado local (${storageKey}):`, error);
    return fallbackState;
  }
}

export function saveWorkspaceState<T>(storageKey: string, state: T): void {
  if (!canUseStorage()) {
    return;
  }

  localStorage.setItem(storageKey, JSON.stringify(state));
}
