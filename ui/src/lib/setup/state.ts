export type SetupDraft = {
    providerName: string;
    model: string;
    workspaceName: string;
    workspaceRoot: string;
    authToken: string;
    executionMode: "local" | "sandboxed";
    backendPriority: string[];
};

export type SetupStepId = "provider" | "workspace" | "auth" | "execution" | "review";

export type SetupStep = {
    id: SetupStepId;
    label: string;
    description: string;
};

export const setupCopy = {
    headline: "Configure provider, workspace, and execution defaults",
    body: "MatrixClaw should move from first launch into the workspace through a legible step-based flow. The runtime owns validation and persistence; the shell should make those decisions clear before the first run."
};

export const defaultSetupDraft: SetupDraft = {
    providerName: "openai-compatible",
    model: "gpt-5.4",
    workspaceName: "default",
    workspaceRoot: "~/workspace",
    authToken: "",
    executionMode: "local",
    backendPriority: ["docker", "boxlite"]
};

export const setupDraftStorageKey = "matrixclaw.setupDraft";
export const setupStepStorageKey = "matrixclaw.setupStep";

export const setupFlow: SetupStep[] = [
    {
        id: "provider",
        label: "Provider",
        description: "Choose the API shape and default model before the app can talk to an agent."
    },
    {
        id: "workspace",
        label: "Workspace",
        description: "Decide which workspace and agent identity the app should open into."
    },
    {
        id: "auth",
        label: "Auth",
        description: "Store the provider token used for the first live run."
    },
    {
        id: "execution",
        label: "Execution",
        description: "Pick the local or sandboxed execution policy and preferred backends."
    },
    {
        id: "review",
        label: "Review",
        description: "Confirm the launch contract before the shell enters the workspace."
    }
];

export const reviewChecklist = [
    "Provider and model are explicit",
    "Workspace root is set before first run",
    "Auth token is present",
    "Execution defaults are visible and editable"
];

export function isSetupStepComplete(step: SetupStepId, draft: SetupDraft): boolean {
    switch (step) {
        case "provider":
            return draft.providerName.trim().length > 0 && draft.model.trim().length > 0;
        case "workspace":
            return draft.workspaceName.trim().length > 0 && draft.workspaceRoot.trim().length > 0;
        case "auth":
            return draft.authToken.trim().length > 0;
        case "execution":
            return draft.executionMode === "local" || draft.executionMode === "sandboxed";
        case "review":
            return reviewChecklist.every((_, index) =>
                isSetupStepComplete(setupFlow[index].id, draft)
            );
    }
}
