import { AnimatePresence, m } from "motion/react";
import type { CSSProperties } from "react";
import { unset, useSend } from "../engine";
import type { Notice } from "../gen/ui/v1/common_pb";
import { Button, IconButton } from "../ui/Button";
import { useTiming } from "../ui/motion";

// The one notice bar at the bottom of the screen: something just done with Undo and its countdown,
// or a line of information. It enters rising and fading in and leaves 20% faster; a new notice
// overlaps the old one. The countdown is a bar shrinking over `remaining_ms` from the first view
// that carried the notice (keyed by its text), a pure CSS animation.
export function NoticeBar({ notice }: { notice: Notice | undefined }) {
	const state = notice?.state ?? unset("ShellView.notice");
	return (
		<div className="notice-dock" aria-live="polite">
			<AnimatePresence initial={false}>{shown(state)}</AnimatePresence>
		</div>
	);
}

function shown(state: Notice["state"]) {
	switch (state.case) {
		case "none":
			return null;
		case "undo":
			return (
				<Bar key={`undo-${state.value.text}`}>
					<span className="sm grow">{state.value.text}</span>
					<UndoButton label={state.value.action} />
					<span
						className="notice-progress"
						style={{ "--ms": `${state.value.remainingMs}ms` } as CSSProperties}
						aria-hidden="true"
					/>
				</Bar>
			);
		case "info":
			return (
				<Bar key={`info-${state.value.text}`}>
					<span className="sm grow">{state.value.text}</span>
					<Dismiss />
				</Bar>
			);
		case undefined:
			return unset("Notice.state");
	}
}

function Bar({ children }: { children: React.ReactNode }) {
	const { enter, exit } = useTiming();
	return (
		<m.div
			className="notice"
			role="status"
			initial={{ opacity: 0, y: 16, scale: 0.97 }}
			animate={{ opacity: 1, y: 0, scale: 1 }}
			exit={{ opacity: 0, y: 8, transition: exit }}
			transition={enter}
		>
			{children}
		</m.div>
	);
}

function UndoButton({ label }: { label: string }) {
	const send = useSend();
	return (
		<Button
			variant="ghost"
			className="notice-action"
			onClick={() => send({ case: "undo", value: {} })}
		>
			{label}
		</Button>
	);
}

function Dismiss() {
	const send = useSend();
	return (
		<IconButton
			icon="close"
			label="Dismiss"
			size="sm"
			onClick={() => send({ case: "dismissNotice", value: {} })}
		/>
	);
}
