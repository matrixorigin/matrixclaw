import { fetchJson } from "$lib/http";

export type GatewayCatalogRecord = {
    name: string;
    health: string;
    enabled_by_agent_count: number;
};

export const GATEWAY_CATALOG_ENDPOINT = "/api/gateway";

export async function fetchGatewayCatalog(): Promise<GatewayCatalogRecord[]> {
    return fetchJson<GatewayCatalogRecord[]>(GATEWAY_CATALOG_ENDPOINT);
}
