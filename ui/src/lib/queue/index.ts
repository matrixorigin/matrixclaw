export type QueueControlKind = "steering" | "follow-up";

export type QueueDeliveryTiming = "next-turn" | "next-run" | "queued";

export type QueueControlState = {
    kind: QueueControlKind;
    submitRoute: string;
    deliveryTiming: QueueDeliveryTiming;
    summary: string;
};

export type QueueControlsView = {
    steering: QueueControlState;
    followUp: QueueControlState;
};

export type QueueSubmissionRequest = {
    kind: QueueControlKind;
    message: string;
};

export type QueueSubmissionResult = {
    accepted: boolean;
    state: QueueControlState;
};

export const queueControlsCopy = {
    steering: "Steering is queued for the next assistant turn.",
    followUp: "Follow-up is deferred until the current run completes."
} as const;

export const queueDeliveryLabels: Record<QueueDeliveryTiming, string> = {
    "next-turn": "Next turn",
    "next-run": "Next run",
    queued: "Queued"
};
