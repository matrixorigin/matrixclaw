import { fetchJson } from "$lib/http";

export type AgentSummary = {
    agent_name: string;
    title: string;
    crown_job: string;
    memory_summary: string;
    memory_signal_count: number;
    pinned_memory_count: number;
    enabled_skills: string[];
    enabled_mcp_servers: string[];
    enabled_gateways: string[];
    binding_count: number;
};

export const AGENTS_DIRECTORY_ENDPOINT = "/api/agents";
export const AGENT_DETAIL_ENDPOINT = "/api/agents/detail";

export function agentRoute(agentName: string): string {
    return `/agents/${encodeURIComponent(agentName)}`;
}

export function agentDetailEndpoint(agentName: string): string {
    const query = new URLSearchParams({ agent: agentName }).toString();
    return `${AGENT_DETAIL_ENDPOINT}?${query}`;
}

export async function fetchAgents(): Promise<AgentSummary[]> {
    return fetchJson<AgentSummary[]>(AGENTS_DIRECTORY_ENDPOINT);
}

export async function fetchAgent(agentName: string): Promise<AgentSummary> {
    return fetchJson<AgentSummary>(agentDetailEndpoint(agentName));
}
