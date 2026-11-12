import { unset } from "../engine";
import type { AccountState, AgentStatus } from "../gen/ui/v1/common_pb";
import { cx } from "./cx";

// The status dot of design/components.css. `run` breathes.
export type Tone = "run" | "wait" | "done" | "err" | "unread" | "idle";

// Decorative unless `label` names it, as when the dot is the only sign of a state.
export function Dot({
	tone,
	label,
	className,
}: {
	tone: Tone;
	label?: string;
	className?: string;
}) {
	const classes = cx("dot", tone, className);
	return label ? (
		<span className={classes} role="img" aria-label={label} />
	) : (
		<span className={classes} aria-hidden="true" />
	);
}

export function agentTone(status: AgentStatus | undefined): Tone {
	switch (status?.state.case) {
		case "running":
			return "run";
		case "idle":
			return "done";
		case "waiting":
			return "wait";
		case "deactivated":
			return "idle";
		case undefined:
			return unset("AgentStatus");
	}
}

// An account's dot and the engine's label for its state.
export function accountStatus(account: AccountState | undefined): {
	tone: Tone;
	label: string;
} {
	switch (account?.state.case) {
		case "synced":
			return { tone: "done", label: account.state.value.label };
		case "signInExpired":
			return { tone: "err", label: account.state.value.label };
		case "syncing":
			return { tone: "run", label: account.state.value.label };
		case undefined:
			return unset("AccountState");
	}
}
