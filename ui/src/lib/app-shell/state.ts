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
        href: "/setup",
        label: "Setup",
        caption: "Provider handshake, workspace binding, and runtime defaults.",
        shortcut: "Cmd-1"
    },
    {
        href: "/workspace",
        label: "Workspace",
        caption: "Conversation surface, file references, and live runtime state.",
        shortcut: "Cmd-2"
    },
    {
        href: "/skills",
        label: "Skills",
        caption: "Installed capabilities and per-agent switches.",
        shortcut: "Cmd-3"
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
        detail: "Configuration decides whether launch continues into setup or hands off to the workspace."
    },
    {
        label: "Fallback",
        detail: "If the runtime is unavailable, the launch screen keeps manual routes exposed."
    }
];

export function describeRoute(pathname: string): string {
    if (pathname.startsWith("/setup")) {
        return "Onboarding";
    }

    if (pathname.startsWith("/workspace")) {
        return "Workspace";
    }

    if (pathname.startsWith("/skills")) {
        return "Skills";
    }

    return "Launch";
}

export function isActiveRoute(pathname: string, href: string): boolean {
    if (href === "/") {
        return pathname === "/";
    }

    return pathname === href || pathname.startsWith(`${href}/`);
}
