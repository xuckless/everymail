import { type KeyboardEvent, type ReactNode, useEffect, useId } from "react";

// A list whose selection the engine holds, as a listbox: the view's selected row is the active
// descendant, and the keyboard sends intents. Up and Down select the neighbouring rendered option
// (by pressing its `[data-select]` control, so the row's own intent is sent), Enter sends `onOpen`.
// Options are <Row option={{ id: optionId(key), selected }}> inside <Rows group>.
export function useOptionIds() {
	const list = useId();
	return (key: string) => `${list}${key}`;
}

export function SelectList({
	label,
	active,
	onOpen,
	className,
	children,
}: {
	label: string;
	// The selected option's element id, if any.
	active: string | undefined;
	onOpen?: () => void;
	className?: string;
	children: ReactNode;
}) {
	// The selection may come from the keyboard; keep the selected row in view.
	useEffect(() => {
		if (active)
			document.getElementById(active)?.scrollIntoView({ block: "nearest" });
	}, [active]);
	return (
		<div
			role="listbox"
			tabIndex={0}
			aria-label={label}
			aria-activedescendant={active}
			className={className}
			onKeyDown={(event) => key(event, onOpen)}
		>
			{children}
		</div>
	);
}

function key(event: KeyboardEvent<HTMLDivElement>, onOpen?: () => void) {
	const step = { ArrowDown: 1, ArrowUp: -1 }[event.key];
	if (step) {
		event.preventDefault();
		const options = [
			...event.currentTarget.querySelectorAll<HTMLElement>("[role=option]"),
		];
		const at = options.findIndex((option) => option.ariaSelected === "true");
		const next = options[at < 0 ? 0 : at + step];
		next?.querySelector<HTMLElement>("[data-select]")?.click();
	} else if (event.key === "Enter" && onOpen) {
		event.preventDefault();
		onOpen();
	}
}
