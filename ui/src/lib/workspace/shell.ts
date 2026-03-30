import type { AgentSummary } from "$lib/agents";
import type { QueueControlsView } from "$lib/queue";

export type WorkspaceAgentSurface = {
    agentName: string;
    heading: string;
    crownJob: string;
    memorySummary: string;
    memorySignalCount: number;
    bindingCount: number;
    enabledSkills: string[];
    enabledMcpServers: string[];
    enabledGateways: string[];
};

export type WorkspaceExecutionSnapshot = {
    modeLabel: string;
    visibleBackends: string[];
    sandboxPriority: string[];
    sandboxFailureMessage: string;
    fallbackPolicy: string;
};

export type WorkspaceDiagnosticsCard = {
    title: string;
    label: string;
    body: string;
    tone: "neutral" | "warning";
};

export type WorkspaceShellDiagnostics = {
    queueCards: WorkspaceDiagnosticsCard[];
    executionCards: WorkspaceDiagnosticsCard[];
};

export function buildWorkspaceAgentSurface(agent: AgentSummary): WorkspaceAgentSurface {
    const heading = agent.title.trim() || agent.agent_name.trim() || "Agent";

    return {
        agentName: agent.agent_name,
        heading,
        crownJob: agent.crown_job,
        memorySummary: agent.memory_summary,
        memorySignalCount: agent.memory_signal_count,
        bindingCount: agent.binding_count,
        enabledSkills: agent.enabled_skills,
        enabledMcpServers: agent.enabled_mcp_servers,
        enabledGateways: agent.enabled_gateways
    };
}

export function buildWorkspaceShellDiagnostics(
    queueView: QueueControlsView,
    execution: WorkspaceExecutionSnapshot
): WorkspaceShellDiagnostics {
    return {
        queueCards: [
            {
                title: "Queue",
                label: `${queueView.steering.deliveryTiming} · ${queueView.followUp.deliveryTiming}`,
                body: `${queueView.steering.summary} ${queueView.followUp.summary}`,
                tone: "neutral"
            }
        ],
        executionCards: [
            {
                title: "Execution posture",
                label: execution.modeLabel,
                body: execution.sandboxPriority.join(" > "),
                tone: "neutral"
            },
            {
                title: "Sandbox policy",
                label: "runtime warning",
                body: execution.sandboxFailureMessage,
                tone: "warning"
            }
        ]
    };
}
