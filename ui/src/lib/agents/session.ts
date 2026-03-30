export type SelectedAgentSession = {
    agentName: string;
    sessionId: string;
};

export const defaultSelectedAgentSession: SelectedAgentSession = {
    agentName: "atlas",
    sessionId: ""
};

export function createSelectedAgentSession(
    agentName: string = defaultSelectedAgentSession.agentName,
    sessionId = ""
): SelectedAgentSession {
    return {
        agentName: agentName.trim() || defaultSelectedAgentSession.agentName,
        sessionId: sessionId.trim()
    };
}

export function normalizeSelectedAgentSession(
    value: Partial<SelectedAgentSession> | null | undefined
): SelectedAgentSession {
    return createSelectedAgentSession(value?.agentName, value?.sessionId);
}
