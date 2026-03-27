export type InstalledSkill = {
    name: string;
    version: string;
    supportTier: "native" | "shimmed";
    source: string;
    description: string;
};

export type EnabledSkillState = {
    agentName: string;
    enabledNames: string[];
};

export const installedSkills: InstalledSkill[] = [
    {
        name: "research",
        version: "1.0.0",
        supportTier: "native",
        source: "Imported from OpenClaw-compatible skill package",
        description: "Reusable prompt + workflow bundle for investigation-heavy tasks."
    },
    {
        name: "lint-bridge",
        version: "0.3.1",
        supportTier: "shimmed",
        source: "JSON-RPC bridge adapter",
        description: "Wraps a subprocess plugin while keeping runtime ownership in MatrixClaw."
    }
];

export const enabledSkillState: EnabledSkillState = {
    agentName: "default",
    enabledNames: ["research"]
};

export function isSkillEnabled(skillName: string, state: EnabledSkillState): boolean {
    return state.enabledNames.includes(skillName);
}

export function toggleSkill(
    skillName: string,
    state: EnabledSkillState
): EnabledSkillState {
    const enabledNames = isSkillEnabled(skillName, state)
        ? state.enabledNames.filter((name) => name !== skillName)
        : [...state.enabledNames, skillName];

    enabledNames.sort();
    return {
        ...state,
        enabledNames
    };
}
