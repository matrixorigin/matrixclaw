import { fetchJson } from "$lib/http";

export type McpCatalogRecord = {
    name: string;
    health: string;
    enabled_by_agent_count: number;
};

export const MCP_CATALOG_ENDPOINT = "/api/mcp";

export async function fetchMcpCatalog(): Promise<McpCatalogRecord[]> {
    return fetchJson<McpCatalogRecord[]>(MCP_CATALOG_ENDPOINT);
}
