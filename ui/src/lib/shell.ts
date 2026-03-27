export type ShellRoute = {
    href: string;
    label: string;
    description: string;
};

export const shellRoutes: ShellRoute[] = [
    {
        href: "/workspace",
        label: "Workspace",
        description: "Chat, files, queue state, and execution details."
    },
    {
        href: "/skills",
        label: "Skills",
        description: "Installed inventory and per-agent enablement."
    },
    {
        href: "/setup",
        label: "Setup",
        description: "Provider, workspace, auth, and execution defaults."
    }
];

export const setupSteps = [
    "Provider",
    "Workspace",
    "Auth",
    "Execution",
    "Review"
];

export const executionPriority = ["docker", "boxlite"] as const;
