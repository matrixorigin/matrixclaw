import { agentRoute } from "$lib/agents";
import type { WorkspaceAgentSurface } from "$lib/workspace/shell";

export type WorkspaceDockNavItem = {
    label: string;
    href: string;
    active?: boolean;
    shortCode: string;
};

export type WorkspaceDockStateRow = {
    label: string;
    value: string;
    tone?: "default" | "primary" | "mcp" | "gateway";
};

export type WorkspaceDockModel = {
    title: string;
    agentToken: string;
    dockCopy: string;
    crownJobSummary: string;
    navItems: WorkspaceDockNavItem[];
    agentState: WorkspaceDockStateRow[];
    capabilityState: WorkspaceDockStateRow[];
};

function summarizeCrownJob(crownJob: string): string {
    return crownJob.trim() || "No crown job configured.";
}

export function buildWorkspaceDockModel(
    agentName: string,
    surface: WorkspaceAgentSurface | null
): WorkspaceDockModel {
    const normalizedAgent = agentName.trim() || "atlas";

    return {
        title: "Workspace Controls",
        agentToken: normalizedAgent,
        dockCopy:
            "Select the active agent, move between control surfaces, and inspect compact runtime context without leaving the workspace.",
        crownJobSummary: summarizeCrownJob(surface?.crownJob ?? ""),
        navItems: [
            {
                label: "Agent Detail",
                href: agentRoute(normalizedAgent),
                active: true,
                shortCode: "AG"
            },
            { label: "Skills", href: "/skills", shortCode: "SK" },
            { label: "MCP", href: "/mcp", shortCode: "MC" },
            { label: "Gateway", href: "/gateway", shortCode: "GW" }
        ],
        agentState: [
            { label: "Crown job", value: "active", tone: "primary" },
            { label: "Memory", value: `${surface?.memorySignalCount ?? 0} signals` },
            { label: "Pinned", value: `${surface?.bindingCount ?? 0} bindings` }
        ],
        capabilityState: [
            {
                label: "Skills",
                value: `${surface?.enabledSkills.length ?? 0} enabled`,
                tone: "primary"
            },
            {
                label: "MCP",
                value: `${surface?.enabledMcpServers.length ?? 0} servers`,
                tone: "mcp"
            },
            {
                label: "Gateway",
                value: `${surface?.enabledGateways.length ?? 0} routes`,
                tone: "gateway"
            }
        ]
    };
}
