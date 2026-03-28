export type ShellRoute = {
    href: string;
    label: string;
    description: string;
};

export const shellRoutes: ShellRoute[] = [
    {
        href: "/workspace",
        label: "Workspace",
        description: "Chat, files, queue state, and runtime details."
    },
    {
        href: "/skills",
        label: "Skills",
        description: "Installed inventory and agent enablement."
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

export function sectionTitleForPath(pathname: string): string {
    if (pathname.startsWith("/setup")) {
        return "Setup";
    }

    if (pathname.startsWith("/skills")) {
        return "Skills";
    }

    if (pathname.startsWith("/workspace")) {
        return "Workspace";
    }

    return "Home";
}
