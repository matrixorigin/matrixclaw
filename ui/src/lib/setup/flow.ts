export type SetupFlowDraft = {
    providerName: string;
    model: string;
    workspaceName: string;
    workspaceRoot: string;
    authToken: string;
    executionMode: "local" | "sandboxed";
    backendPriority: string[];
};

export type SetupStepId = "provider" | "workspace" | "auth" | "execution" | "review";

export type SetupStepDefinition = {
    id: SetupStepId;
    title: string;
    eyebrow: string;
    description: string;
    detail: string;
};

export const setupFlowSteps: SetupStepDefinition[] = [
    {
        id: "provider",
        title: "Model provider",
        eyebrow: "Step 1",
        description: "Choose the provider bridge and the default model used by the desktop shell.",
        detail: "This controls the runtime handshake that the app performs before the workspace opens."
    },
    {
        id: "workspace",
        title: "Workspace binding",
        eyebrow: "Step 2",
        description: "Bind the desktop shell to the first workspace root and visible workspace name.",
        detail: "The setup contract persists a stable root so file references stay deterministic."
    },
    {
        id: "auth",
        title: "Access token",
        eyebrow: "Step 3",
        description: "Store the token that unlocks authenticated provider calls from the runtime.",
        detail: "The shell should make missing auth obvious before the user reaches the composer."
    },
    {
        id: "execution",
        title: "Execution defaults",
        eyebrow: "Step 4",
        description: "Pick the default execution lane and keep fallback behavior legible.",
        detail: "This step exists so the desktop shell exposes runtime posture instead of hiding it."
    },
    {
        id: "review",
        title: "Review and write",
        eyebrow: "Step 5",
        description: "Confirm the launch contract, persist config, and hand off to the workspace.",
        detail: "The setup flow should end with one explicit write to `/api/setup/config`."
    }
];

export function stepIssues(draft: SetupFlowDraft, stepId: SetupStepId): string[] {
    const trimmedToken = draft.authToken.trim();

    switch (stepId) {
        case "provider":
            return [
                ...(draft.providerName.trim() ? [] : ["Provider name is required."]),
                ...(draft.model.trim() ? [] : ["Model id is required."])
            ];
        case "workspace":
            return [
                ...(draft.workspaceName.trim() ? [] : ["Workspace name is required."]),
                ...(draft.workspaceRoot.trim() ? [] : ["Workspace root is required."])
            ];
        case "auth":
            return trimmedToken ? [] : ["Auth token is required."];
        case "execution":
            return draft.backendPriority.length > 0
                ? []
                : ["At least one backend priority label must be visible."];
        case "review":
            return [
                ...stepIssues(draft, "provider"),
                ...stepIssues(draft, "workspace"),
                ...stepIssues(draft, "auth"),
                ...stepIssues(draft, "execution")
            ];
    }
}

export function stepIsComplete(draft: SetupFlowDraft, stepId: SetupStepId): boolean {
    return stepIssues(draft, stepId).length === 0;
}

export function completedStepCount(draft: SetupFlowDraft): number {
    return setupFlowSteps.filter((step) => step.id !== "review" && stepIsComplete(draft, step.id))
        .length;
}

export function maskedToken(token: string): string {
    const trimmed = token.trim();

    if (!trimmed) {
        return "Not provided";
    }

    if (trimmed.length <= 8) {
        return `${trimmed.slice(0, 2)}****`;
    }

    return `${trimmed.slice(0, 4)}...${trimmed.slice(-4)}`;
}

export function buildSetupPayload(draft: SetupFlowDraft) {
    return {
        provider: {
            provider_name: draft.providerName.trim(),
            model: draft.model.trim()
        },
        workspace: {
            name: draft.workspaceName.trim(),
            root: draft.workspaceRoot.trim()
        },
        auth: {
            token: draft.authToken.trim()
        },
        execution: {
            mode: draft.executionMode === "sandboxed" ? "Sandboxed" : "Local",
            backend:
                draft.executionMode === "sandboxed"
                    ? {
                          kind: "Sandbox",
                          label: "sandbox",
                          requires_docker: false
                      }
                    : {
                          kind: "LocalCommand",
                          label: "local-command",
                          requires_docker: false
                      }
        }
    };
}
