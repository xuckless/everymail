import { AnimatePresence, m, usePresence } from "motion/react";
import type { ReactNode } from "react";
import { cx } from "./cx";
import { move } from "./motion";
import { stagger } from "./stagger";

// A list whose rows enter with the 30ms stagger (`.row-in.stagger`), leave with `.row-out`, and
// glide to their new place when rows before them leave or reorder (Motion layout, by stable id).
// Give each <Row> the row's stable id as its key and its index in the list. Inside a <SelectList>
// the rows are a group of its options: pass `group` here and `option` to each row.
export function Rows({
	label,
	group,
	className,
	children,
}: {
	label: string;
	group?: boolean;
	className?: string;
	children: ReactNode;
}) {
	return (
		<ul
			className={cx("rows", className)}
			aria-label={label}
			role={group ? "group" : undefined}
		>
			<AnimatePresence mode="popLayout">{children}</AnimatePresence>
		</ul>
	);
}

export function Row({
	index,
	option,
	className,
	children,
}: {
	index: number;
	// An option of a SelectList: its element id and whether the view selects it.
	option?: { id: string; selected: boolean };
	className?: string;
	children: ReactNode;
}) {
	return (
		<m.li
			layout="position"
			transition={move}
			role={option ? "option" : undefined}
			id={option?.id}
			aria-selected={option?.selected}
		>
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
			className={cx(className, present ? "row-in stagger" : "row-out")}
			style={stagger(index)}
			onAnimationEnd={(event) => {
				if (!present && event.target === event.currentTarget) safeToRemove();
			}}
		>
			{children}
		</div>
	);
}
