export async function fetchJson<T>(input: RequestInfo | URL, init?: RequestInit): Promise<T> {
    const headers = new Headers(init?.headers);
    if (!headers.has("content-type")) {
        headers.set("content-type", "application/json");
    }

    const response = await fetch(input, {
        ...init,
        headers
    });

    if (!response.ok) {
        const body = await response.text();
        throw new Error(body || `request failed with status ${response.status}`);
    }

    return (await response.json()) as T;
}

export function errorMessage(error: unknown): string {
    if (error instanceof Error) {
        return error.message;
    }

    return String(error);
}
