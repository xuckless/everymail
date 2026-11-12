import { AnimatePresence, m, usePresence } from "motion/react";
import type { ReactNode } from "react";
import { cx } from "./cx";
import { useTiming } from "./motion";
import { stagger } from "./stagger";

// A list whose rows enter with the 30ms stagger (`.row-in`, delayed by `stagger`), leave with `.row-out`, and
// glide to their new place when rows before them leave or reorder (Motion layout, by stable id).
// Give each <Row> the row's stable id as its key and its index in the list.
export function Rows({
	label,
	className,
	children,
}: {
	label: string;
	className?: string;
	children: ReactNode;
}) {
	return (
		<ul className={cx("rows", className)} aria-label={label}>
			<AnimatePresence mode="popLayout">{children}</AnimatePresence>
		</ul>
	);
}

export function Row({
	index,
	className,
	children,
}: {
	index: number;
	className?: string;
	children: ReactNode;
}) {
	const { move } = useTiming();
	return (
		<m.li layout="position" transition={move}>
			<Face index={index} className={className}>
				{children}
			</Face>
		</m.li>
	);
}

function Face({
	index,
	className,
	children,
}: {
	index: number;
	className?: string;
	children: ReactNode;
}) {
	const [present, safeToRemove] = usePresence();
	return (
		<div
			className={cx(className, present ? "row-in" : "row-out")}
			style={stagger(index)}
			onAnimationEnd={(event) => {
				if (!present && event.target === event.currentTarget) safeToRemove();
			}}
		>
			{children}
		</div>
	);
}
