import { usePresence } from "motion/react";
import {
	type AnimationEvent,
	type CSSProperties,
	type ReactNode,
	useEffect,
	useRef,
} from "react";
import { type OverlayPhase, useOverlay } from "../presentation";
import { cx } from "./cx";

// Two kinds of overlay, one look: the design's `.in`/`.out` classes (enter ease-out from scale
// 0.95 or from an edge, exit 20% shorter).
//   The user opens it (Popover, UserSheet): its phase is the OverlayPhase of presentation.ts.
//   The engine opens it (PresentSheet, PresentDrawer, PresentPopover): render it keyed inside
//   Motion's <AnimatePresence> while the view says so; it leaves the way it came when the view
//   stops saying so, and Escape or a press outside sends `onClose` (the engine's close intent).

const motionOf = (phase: OverlayPhase) => (phase === "closing" ? "out" : "in");

// Only the overlay's own animation settles it, not one bubbling up from a staggered row inside.
const own = (event: AnimationEvent, then: () => void) => {
	if (event.target === event.currentTarget) then();
};

// Escape, and a press outside what `inside` accepts, while `active`.
function useDismiss(
	active: boolean,
	dismiss: () => void,
	inside: (target: Element) => boolean,
) {
	useEffect(() => {
		if (!active) return;
		const press = (event: PointerEvent) => {
			if (!inside(event.target as Element)) dismiss();
		};
		const key = (event: KeyboardEvent) => {
			if (event.key === "Escape") dismiss();
		};
		document.addEventListener("pointerdown", press);
		document.addEventListener("keydown", key);
		return () => {
			document.removeEventListener("pointerdown", press);
			document.removeEventListener("keydown", key);
		};
	}, [active, dismiss, inside]);
}

const focusable =
	'button:not(:disabled), [href], input:not(:disabled), select:not(:disabled), textarea:not(:disabled), [tabindex]:not([tabindex="-1"])';

// Keyboard for a menu or popover dialog (WAI-ARIA menu pattern), on the DOM it renders: opening
// moves focus to the checked item (or the first); ArrowUp, ArrowDown, Home and End move between
// items; closing returns focus to the trigger when it was inside. Nothing is kept: focus is read
// from the document each time.
function usePopoverFocus(
	shown: boolean,
	anchor: { current: HTMLElement | null },
	role: "menu" | "dialog",
) {
	useEffect(() => {
		const root = anchor.current;
		if (!shown || !root) return;
		const trigger = root.querySelector<HTMLElement>("[aria-haspopup]");
		const items = () => [
			...root.querySelectorAll<HTMLElement>(
				role === "menu"
					? '.pop [role^="menuitem"]:not(:disabled)'
					: `.pop :is(${focusable})`,
			),
		];
		const first = items().find(
			(item) => item.getAttribute("aria-checked") === "true",
		);
		(first ?? items()[0])?.focus();
		const key = (event: KeyboardEvent) => {
			const all = items();
			const at = all.indexOf(document.activeElement as HTMLElement);
			const to = (index: number) => {
				event.preventDefault();
				all[(index + all.length) % all.length]?.focus();
			};
			switch (event.key) {
				case "ArrowDown":
					return to(at + 1);
				case "ArrowUp":
					return to(at - 1);
				case "Home":
					return to(0);
				case "End":
					return to(all.length - 1);
			}
		};
		root.addEventListener("keydown", key);
		return () => {
			root.removeEventListener("keydown", key);
			const active = document.activeElement;
			if (active === document.body || root.contains(active)) trigger?.focus();
		};
	}, [shown, anchor, role]);
}

// A modal sheet or drawer (aria-modal): focus moves in when it opens, Tab and Shift+Tab stay
// inside, everything outside it is inert, Escape calls `onEscape`, and closing gives focus back to
// where it was.
function useModal(
	dialog: { current: HTMLElement | null },
	onEscape?: () => void,
) {
	useEffect(() => {
		const root = dialog.current;
		if (!root) return;
		const before = document.activeElement as HTMLElement | null;
		const outside: HTMLElement[] = [];
		for (
			let at: HTMLElement | null = root;
			at && at !== document.body;
			at = at.parentElement
		) {
			for (const sibling of at.parentElement?.children ?? []) {
				if (
					sibling !== at &&
					sibling instanceof HTMLElement &&
					!sibling.inert &&
					!sibling.classList.contains("scrim")
				) {
					sibling.inert = true;
					outside.push(sibling);
				}
			}
		}
		root.querySelector<HTMLElement>(focusable)?.focus();
		const key = (event: KeyboardEvent) => {
			if (event.key !== "Tab") return;
			const all = [...root.querySelectorAll<HTMLElement>(focusable)];
			const first = all[0];
			const last = all.at(-1);
			if (event.shiftKey && document.activeElement === first) {
				event.preventDefault();
				last?.focus();
			} else if (!event.shiftKey && document.activeElement === last) {
				event.preventDefault();
				first?.focus();
			}
		};
		root.addEventListener("keydown", key);
		return () => {
			root.removeEventListener("keydown", key);
			for (const element of outside) element.inert = false;
			before?.focus();
		};
	}, [dialog]);
	useEffect(() => {
		const root = dialog.current;
		if (!root || !onEscape) return;
		const key = (event: KeyboardEvent) => {
			if (event.key === "Escape") onEscape();
		};
		root.addEventListener("keydown", key);
		return () => root.removeEventListener("keydown", key);
	}, [dialog, onEscape]);
}

export type TriggerProps = {
	onClick: () => void;
	"aria-expanded": boolean;
	"aria-haspopup": "menu" | "dialog";
};

// A menu that enters from and exits to its trigger: below the trigger's `align` corner (`stretch`
// spans the trigger's width), scaling from there. Outside press and Escape dismiss it; `children`
// gets `dismiss` for items that act and close. Items are menuitem, or menuitemradio for a choice.
export function Popover({
	label,
	align = "start",
	side = "below",
	role = "menu",
	className,
	trigger,
	children,
}: {
	label: string;
	align?: "start" | "end" | "stretch";
	// Above for a trigger near the bottom of the screen, such as the composer's.
	side?: "below" | "above";
	// A confirmation is a dialog, not a menu.
	role?: "menu" | "dialog";
	className?: string;
	trigger: (props: TriggerProps) => ReactNode;
	children: (dismiss: () => void) => ReactNode;
}) {
	const { phase, dispatch } = useOverlay();
	const anchor = useRef<HTMLSpanElement>(null);
	const shown = phase !== "closed";
	useDismiss(
		shown,
		() => dispatch("dismiss"),
		(target) => anchor.current?.contains(target) ?? false,
	);
	usePopoverFocus(shown, anchor, role);
	return (
		<span
			ref={anchor}
			className={cx("pop-anchor", align === "stretch" && "pop-anchor-wide")}
		>
			{trigger({
				onClick: () => dispatch("toggle"),
				"aria-expanded": phase === "opening" || phase === "open",
				"aria-haspopup": role,
			})}
			{shown && (
				// biome-ignore lint/a11y/useAriaPropsSupportedByRole: the role is menu or dialog, both of which take a label
				<div
					className={cx(
						"pop",
						motionOf(phase),
						`pop-${align}`,
						side === "above" && "pop-above",
						className,
					)}
					role={role}
					aria-label={label}
					onAnimationEnd={(event) => own(event, () => dispatch("settled"))}
				>
					{children(() => dispatch("dismiss"))}
				</div>
			)}
		</span>
	);
}

type SheetProps = {
	label: string;
	// The scrim. Engine-opened sheets send their back or close intent here.
	onDismiss?: () => void;
	children: ReactNode;
};

// The phone's bottom sheet over a scrim. Its positioned ancestor is the phone frame.
function Sheet({
	label,
	motion,
	onDismiss,
	onSettled,
	children,
}: SheetProps & { motion: "in" | "out"; onSettled: () => void }) {
	const dialog = useRef<HTMLDivElement>(null);
	useModal(dialog, onDismiss);
	return (
		<>
			<div className={cx("scrim", motion)} onClick={onDismiss} aria-hidden />
			<div
				ref={dialog}
				className={cx("sheet", motion)}
				role="dialog"
				aria-modal="true"
				aria-label={label}
				onAnimationEnd={(event) => own(event, onSettled)}
			>
				<div className="sheet-body">
					<span className="grabber" />
					{children}
				</div>
			</div>
		</>
	);
}

// A sheet the user opens: `const sheet = useOverlay()`, a trigger calling
// `sheet.dispatch("toggle")`, and `<UserSheet overlay={sheet} …>`.
export function UserSheet({
	overlay,
	...props
}: SheetProps & { overlay: ReturnType<typeof useOverlay> }) {
	const { phase, dispatch } = overlay;
	if (phase === "closed") return null;
	return (
		<Sheet
			{...props}
			motion={motionOf(phase)}
			onDismiss={() => dispatch("dismiss")}
			onSettled={() => dispatch("settled")}
		/>
	);
}

// The engine-opened overlay's motion: in while present, out while leaving, removed once out.
function usePresent() {
	const [present, safeToRemove] = usePresence();
	return {
		present,
		motion: present ? ("in" as const) : ("out" as const),
		settled: () => {
			if (!present) safeToRemove();
		},
	};
}

export function PresentSheet(props: SheetProps) {
	const { motion, settled } = usePresent();
	return <Sheet {...props} motion={motion} onSettled={settled} />;
}

const nowhere = () => false;

// The event drawer, from the right edge over a scrim.
export function PresentDrawer({
	label,
	onClose,
	className,
	children,
}: {
	label: string;
	onClose: () => void;
	className?: string;
	children: ReactNode;
}) {
	const { present, motion, settled } = usePresent();
	const dialog = useRef<HTMLDivElement>(null);
	useDismiss(present, onClose, nowhere);
	useModal(dialog);
	return (
		<>
			<div className={cx("scrim", motion)} onClick={onClose} aria-hidden />
			<div
				ref={dialog}
				className={cx("drawer-r", motion, className)}
				role="dialog"
				aria-modal="true"
				aria-label={label}
				onAnimationEnd={(event) => own(event, settled)}
			>
				{children}
			</div>
		</>
	);
}

// A popover the engine opens from its trigger, placed by the caller's class and style. A press
// inside `keep` (a selector: the popover and the triggers whose own click toggles it) keeps it.
export function PresentPopover({
	label,
	keep,
	className,
	style,
	onClose,
	children,
}: {
	label: string;
	keep: string;
	className?: string;
	style?: CSSProperties;
	onClose: () => void;
	children: ReactNode;
}) {
	const { present, motion, settled } = usePresent();
	useDismiss(present, onClose, (target) => target.closest(keep) !== null);
	return (
		<div
			className={cx("pop", motion, className)}
			role="dialog"
			aria-label={label}
			style={style}
			onAnimationEnd={(event) => own(event, settled)}
		>
			{children}
		</div>
	);
}
