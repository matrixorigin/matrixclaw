export type SetupDraft = {
    providerName: string;
    model: string;
    workspaceName: string;
    workspaceRoot: string;
    authToken: string;
    executionMode: "local" | "sandboxed";
    backendPriority: string[];
};

export const setupCopy = {
    headline: "Configure provider, workspace, and execution defaults",
    body: "MatrixClaw should move from first-launch setup into the workspace with one validated submission. The Rust runtime owns validation and persistence; the browser flow should make those choices legible."
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

export const reviewChecklist = [
    "Provider and model are explicit",
    "Workspace root is set before first run",
    "Auth token is present",
    "Execution defaults are visible and editable"
];
