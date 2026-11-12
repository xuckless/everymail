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

export type TriggerProps = {
	onClick: () => void;
	"aria-expanded": boolean;
	"aria-haspopup": "menu";
};

// A menu that enters from and exits to its trigger: below the trigger's `align` corner (`stretch`
// spans the trigger's width), scaling from there. Outside press and Escape dismiss it; `children`
// gets `dismiss` for items that act and close. Items are menuitem, or menuitemradio for a choice.
export function Popover({
	label,
	align = "start",
	className,
	trigger,
	children,
}: {
	label: string;
	align?: "start" | "end" | "stretch";
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
	return (
		<span
			ref={anchor}
			className={cx("pop-anchor", align === "stretch" && "pop-anchor-wide")}
		>
			{trigger({
				onClick: () => dispatch("toggle"),
				"aria-expanded": phase === "opening" || phase === "open",
				"aria-haspopup": "menu",
			})}
			{shown && (
				<div
					className={cx("pop", motionOf(phase), `pop-${align}`, className)}
					role="menu"
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
	return (
		<>
			<div className={cx("scrim", motion)} onClick={onDismiss} aria-hidden />
			<div
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
	useDismiss(present, onClose, nowhere);
	return (
		<>
			<div className={cx("scrim", motion)} onClick={onClose} aria-hidden />
			<div
				className={cx("drawer-r", motion, className)}
				role="dialog"
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
