export type AppShellNavItem = {
    href: string;
    label: string;
    caption: string;
    shortcut: string;
};

export type ShellSignal = {
    label: string;
    value: string;
    tone: "neutral" | "accent" | "warn";
};

export type BootNote = {
    label: string;
    detail: string;
};

export const appShellNav: AppShellNavItem[] = [
    {
        href: "/workspace",
        label: "Workspace",
        caption: "Talk to the selected agent and monitor the current run.",
        shortcut: "Cmd-1"
    },
    {
        href: "/agents",
        label: "Agents",
        caption: "Browse agents and open per-agent configuration.",
        shortcut: "Cmd-2"
    },
    {
        href: "/skills",
        label: "Skills",
        caption: "Manage the global skills catalog.",
        shortcut: "Cmd-3"
    },
    {
        href: "/mcp",
        label: "MCP",
        caption: "Inspect shared MCP servers and connection health.",
        shortcut: "Cmd-4"
    },
    {
        href: "/gateway",
        label: "Gateway",
        caption: "Inspect global messaging gateways and routing posture.",
        shortcut: "Cmd-5"
    }
];

export const shellSignals: ShellSignal[] = [
    { label: "Window", value: "Desktop shell", tone: "accent" },
    { label: "Runtime", value: "Loopback bridge", tone: "neutral" },
    { label: "Safety", value: "Local-first", tone: "warn" }
];

export const bootNotes: BootNote[] = [
    {
        label: "Bootstrap",
        detail: "The shell probes `/healthz` before opening the first working surface."
    },
    {
        label: "State path",
        detail: "Configuration decides whether launch continues into the product shell or hands off to the workspace."
    },
    {
        label: "Fallback",
        detail: "If the runtime is unavailable, the launch screen keeps manual routes exposed."
    }
];

export function describeRoute(pathname: string): string {
    if (pathname.startsWith("/setup")) {
        return "Setup";
    }

    const matchedRoute = appShellNav.find((route) => isActiveRoute(pathname, route.href));
    if (matchedRoute) {
        return matchedRoute.label;
    }

    return "Launch";
}

export function isActiveRoute(pathname: string, href: string): boolean {
    if (href === "/") {
        return pathname === "/";
    }

    return pathname === href || pathname.startsWith(`${href}/`);
}
