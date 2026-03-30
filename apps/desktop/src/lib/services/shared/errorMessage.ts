function isRecord(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === 'object' && !Array.isArray(value);
}

function resolveSerializedErrorMessage(value: unknown): string | null {
  if (typeof value !== 'string') return null;

  const trimmed = value.trim();
  if (!trimmed || trimmed === '[object Object]') return null;

  try {
    return extractServiceErrorMessage(JSON.parse(trimmed), trimmed);
  } catch {
    return trimmed;
  }
}

export function extractServiceErrorMessage(error: unknown, fallback: string): string {
  if (typeof error === 'string') {
    return resolveSerializedErrorMessage(error) ?? fallback;
  }

  if (error instanceof Error) {
    return resolveSerializedErrorMessage(error.message) ?? fallback;
  }

  if (isRecord(error)) {
    const message = resolveSerializedErrorMessage(error.message) ?? resolveSerializedErrorMessage(error.error);
    if (message) return message;
  }

  return fallback;
}
