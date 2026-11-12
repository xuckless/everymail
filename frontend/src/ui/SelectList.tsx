import { type ReactNode, useEffect, useRef } from "react";
import { useSend } from "../engine";

// A list whose selection the engine holds. It is one tab stop: only the selected row's control (its
// `[data-select]` button, or the first row's while none is selected) is tabbable, see `tabStop`.
// Down and Up send Intent.select_next and select_previous; Enter sends `onOpen`. When the engine's
// new selection arrives with focus in the list, focus follows it. Rows keep their hover actions
// out of the tab order: the reading pane's toolbar has the same.
export function SelectList({
	active,
	onOpen,
	children,
}: {
	// The selected row's key, if any.
	active: string | undefined;
	onOpen?: () => void;
	children: ReactNode;
}) {
	const send = useSend();
	const list = useRef<HTMLDivElement>(null);
	// biome-ignore lint/correctness/useExhaustiveDependencies: runs when the selection changes
	useEffect(() => {
		const current = list.current?.querySelector<HTMLElement>(
			'[data-select][aria-current="true"]',
		);
		current?.scrollIntoView({ block: "nearest" });
		if (list.current?.contains(document.activeElement)) current?.focus();
	}, [active]);
	return (
		// biome-ignore lint/a11y/noStaticElementInteractions: keys bubble up from the rows' buttons; the wrapper itself takes no focus
		<div
			ref={list}
			onKeyDown={(event) => {
				switch (event.key) {
					case "ArrowDown":
						event.preventDefault();
						return send({ case: "selectNext", value: {} });
					case "ArrowUp":
						event.preventDefault();
						return send({ case: "selectPrevious", value: {} });
					case "Enter":
						if (!onOpen) return;
						event.preventDefault();
						return onOpen();
				}
			}}
		>
			{children}
		</div>
	);
}

// Whether a row's control is the list's one tab stop.
export const tabStop = (
	selected: boolean,
	index: number,
	anySelected: boolean,
) => selected || (!anySelected && index === 0);
