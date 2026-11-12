import { createContext, useContext } from "react";
import type { ActionOutcome, ActionTarget } from "../gen/ui/v1/common_pb";
import { cx } from "./cx";

// The one presentation of what became of the user's latest actions (ShellView.outcomes). The
// engine keeps an outcome for about two seconds: a control that sees `done` pulses once (motion
// rule 8), one that sees `failed` shows the engine's reason under it. Shell.tsx provides the list.
export const OutcomesContext = createContext<ActionOutcome[]>([]);

export type Target = ActionTarget["target"];

export type Outcome =
	| { kind: "none" }
	| { kind: "done"; text: string }
	| { kind: "failed"; reason: string };

export function useOutcome(target: Target): Outcome {
	const found = useContext(OutcomesContext).find(
		(outcome) =>
			outcome.target?.target.case === target.case &&
			outcome.target?.target.value === target.value,
	);
	switch (found?.state.case) {
		case undefined:
			return { kind: "none" };
		case "done":
			return { kind: "done", text: found.state.value.text };
		case "failed":
			return { kind: "failed", reason: found.state.value.reason };
	}
}

// The class a control wears for its outcome: the success pulse.
export const pulseOn = (outcome: Outcome) =>
	outcome.kind === "done" ? "pulse" : undefined;

// The outcome's line under a control or form: the failure reason, or the done text.
export function OutcomeNote({
	outcome,
	className,
}: {
	outcome: Outcome;
	className?: string;
}) {
	switch (outcome.kind) {
		case "none":
			return null;
		case "done":
			return (
				<span className={cx("xs ok settle", className)} role="status">
					{outcome.text}
				</span>
			);
		case "failed":
			return (
				<span className={cx("xs bad settle", className)} role="alert">
					{outcome.reason}
				</span>
			);
	}
}
