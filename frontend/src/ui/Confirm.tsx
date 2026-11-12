import type { ReactNode } from "react";
import { Button } from "./Button";
import { Popover, type TriggerProps } from "./Overlay";

// The one confirmation: a popover from its trigger asking the question, with Cancel and the
// action. Used for every destructive control (delete, disconnect, discard, reset).
export function Confirm({
	question,
	action,
	onConfirm,
	align = "start",
	side = "below",
	trigger,
}: {
	question: string;
	action: string;
	onConfirm: () => void;
	align?: "start" | "end";
	side?: "below" | "above";
	trigger: (props: TriggerProps) => ReactNode;
}) {
	return (
		<Popover
			label={question}
			align={align}
			side={side}
			role="dialog"
			className="confirm"
			trigger={trigger}
		>
			{(dismiss) => (
				<div className="confirm-body">
					<span className="sm">{question}</span>
					<span className="confirm-actions">
						<Button variant="ghost" onClick={dismiss}>
							Cancel
						</Button>
						<Button
							variant="primary"
							className="danger"
							onClick={() => {
								dismiss();
								onConfirm();
							}}
						>
							{action}
						</Button>
					</span>
				</div>
			)}
		</Popover>
	);
}
