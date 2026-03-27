export type ExecutionBackendLabel = "local" | "docker" | "boxlite";

export const visibleExecutionBackends: ExecutionBackendLabel[] = [
    "local",
    "docker",
    "boxlite"
];

export const sandboxPriority: ExecutionBackendLabel[] = ["docker", "boxlite"];

export const sandboxFailureMessage = "sandbox required but unavailable";
