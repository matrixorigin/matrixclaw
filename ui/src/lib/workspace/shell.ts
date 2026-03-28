export type QueueControlView = {
    kind: "steering" | "follow-up";
    submitRoute: string;
    deliveryTiming: "next-turn" | "next-run" | "queued";
    summary: string;
};

export type QueueControlsPanelView = {
    steering: QueueControlView;
    followUp: QueueControlView;
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

export function buildWorkspaceShellDiagnostics(
    queueView: QueueControlsPanelView,
    execution: WorkspaceExecutionSnapshot
): WorkspaceShellDiagnostics {
    return {
        queueCards: [
            {
                title: "Steering queue",
                label: queueView.steering.deliveryTiming,
                body: queueView.steering.summary,
                tone: "neutral"
            },
            {
                title: "Follow-up queue",
                label: queueView.followUp.deliveryTiming,
                body: queueView.followUp.summary,
                tone: "neutral"
            }
        ],
        executionCards: [
            {
                title: "Visible backends",
                label: execution.modeLabel,
                body: execution.visibleBackends.join(", "),
                tone: "neutral"
            },
            {
                title: "Sandbox priority",
                label: execution.fallbackPolicy,
                body: execution.sandboxPriority.join(" > "),
                tone: "neutral"
            },
            {
                title: "Sandbox policy",
                label: "required failures stay explicit",
                body: execution.sandboxFailureMessage,
                tone: "warning"
            }
        ]
    };
}
