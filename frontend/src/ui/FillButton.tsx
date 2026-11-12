import { unset } from "../engine";
import type { ApprovalState } from "../gen/ui/v1/common_pb";
import { cx } from "./cx";

// Whether the engine reports the approved action done.
export function approvalDone(state: ApprovalState | undefined) {
	switch (state?.state.case) {
		case "waiting":
			return false;
		case "done":
			return true;
		case undefined:
			return unset("ApprovalState");
	}
}

// The design's approve button: when the engine reports the action done it fills, its label
// crossfades to `doneLabel` and it pulses once. No intent approves anything yet.
export function FillButton({
	done,
	label,
	doneLabel,
	className,
}: {
	done: boolean;
	label: string;
	doneLabel: string;
	className?: string;
}) {
	return (
		<button
			type="button"
			className={cx(
				"btn primary press fillbtn",
				done && "done pulse",
				className,
			)}
		>
			<span className="fill" />
			<span className="swap">
				<span className={cx(done && "hide")}>{label}</span>
				<span className={cx(!done && "hide")}>{doneLabel}</span>
			</span>
		</button>
	);
}
