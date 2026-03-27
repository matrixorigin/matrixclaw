export type WorkspaceEntryKind = "file" | "directory";

export type WorkspaceEntry = {
    relativePath: string;
    kind: WorkspaceEntryKind;
    referenceToken: string;
};

export type WorkspaceExplorerContract = {
    filesRoute: string;
    referenceRoute: string;
};

export const workspaceExplorerContract: WorkspaceExplorerContract = {
    filesRoute: "/api/workspace/files",
    referenceRoute: "/api/workspace/reference"
};

export function formatWorkspaceReference(relativePath: string): string {
    const normalized = relativePath
        .replace(/\\/g, "/")
        .replace(/^\/+/, "")
        .replace(/\/{2,}/g, "/");
    return `[[workspace:${normalized}]]`;
}
