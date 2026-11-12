import { unset, useSend } from "../engine";
import type { ApprovalState } from "../gen/ui/v1/common_pb";
import { Button } from "./Button";
import { cx } from "./cx";
import type { Tone } from "./Dot";
import { pulseOn, useOutcome } from "./Outcome";

// What the engine reports of an approval: still waiting, being carried out, carried out, or
// declined (with why). Its dot and whether its actions still apply.
export type Approval =
	| { kind: "waiting" }
	| { kind: "resolving" }
	| { kind: "done" }
	| { kind: "declined"; reason: string };

export function approvalOf(state: ApprovalState | undefined): Approval {
	switch (state?.state.case) {
		case "waiting":
			return { kind: "waiting" };
		case "resolving":
			return { kind: "resolving" };
		case "done":
			return { kind: "done" };
		case "declined":
			return { kind: "declined", reason: state.state.value.reason };
		case undefined:
			return unset("ApprovalState");
	}
}

export function approvalTone(approval: Approval): Tone {
	switch (approval.kind) {
		case "waiting":
			return "wait";
		case "resolving":
			return "run";
		case "done":
			return "done";
		case "declined":
			return "idle";
	}
}

// An approval item's actions, wherever it shows (Approvals, a thread, the reading pane). A reply to
// send: Approve and send, Edit (in the composer, where sending approves it) and Decline. A list to
// leave: Unsubscribe and Keep. They apply while it waits. Approve fills and its label crossfades
// when the engine reports it done, and pulses once on that fresh success. A declined item shows
// the reason in place of its actions.
export function ApprovalActions({
	approvalId,
	state,
	action,
	size,
}: {
	approvalId: string;
	state: ApprovalState | undefined;
	action: "send" | "unsubscribe";
	// lg: the phone's touch size.
	size?: "lg";
}) {
	const send = useSend();
	const approval = approvalOf(state);
	const outcome = useOutcome({ case: "approvalId", value: approvalId });
	const [approve, approved, decline] = labels(action);
	if (approval.kind === "declined")
		return <span className="sm muted settle">{approval.reason}</span>;
	const done = approval.kind === "done";
	const closed = approval.kind !== "waiting";
	return (
		<>
			<button
				type="button"
				disabled={closed}
				aria-busy={approval.kind === "resolving"}
				onClick={() => send({ case: "approveItem", value: { approvalId } })}
				className={cx(
					"btn primary press fillbtn",
					size,
					done && "done",
					pulseOn(outcome),
				)}
			>
				<span className="fill" />
				<span className="swap">
					<span className={cx(done && "hide")}>{approve}</span>
					<span className={cx(!done && "hide")}>{approved}</span>
				</span>
			</button>
			{action === "send" && (
				<Button
					variant="soft"
					size={size}
					disabled={closed}
					onClick={() => send({ case: "editApproval", value: { approvalId } })}
				>
					Edit
				</Button>
			)}
			<Button
				variant="ghost"
				size={size}
				disabled={closed}
				onClick={() => send({ case: "declineItem", value: { approvalId } })}
			>
				{decline}
			</Button>
		</>
	);
}

function labels(action: "send" | "unsubscribe") {
	switch (action) {
		case "send":
			return ["Approve and send", "Sent", "Decline"] as const;
		case "unsubscribe":
			return ["Unsubscribe", "Done", "Keep"] as const;
	}
}
