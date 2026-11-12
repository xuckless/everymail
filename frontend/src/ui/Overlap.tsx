import { AnimatePresence, m } from "motion/react";
import type { ReactNode } from "react";
import { cx } from "./cx";
import { enter, exit } from "./motion";

// Old and new states overlap: when `id` changes, the old content fades out with a slight blur
// while the new fades in, in the same grid cell (`.stack`). Key it by what the content shows: a
// thread id, a oneof case. Motion keeps the old content mounted until it has left.
const hidden = { opacity: 0, y: 4, filter: "blur(2px)" };
const shown = { opacity: 1, y: 0, filter: "blur(0px)" };

export function Overlap({
	id,
	className,
	children,
}: {
	id: string;
	className?: string;
	children: ReactNode;
}) {
	return (
		<div className={cx("stack", className)}>
			<AnimatePresence initial={false}>
				<m.div
					key={id}
					className="overlap-layer"
					initial={hidden}
					animate={shown}
					exit={{ ...hidden, pointerEvents: "none", transition: exit }}
					transition={enter}
				>
					{children}
				</m.div>
			</AnimatePresence>
		</div>
	);
}
