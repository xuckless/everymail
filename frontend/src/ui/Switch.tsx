import { m } from "motion/react";
import { cx } from "./cx";
import { move } from "./motion";

// The design's `.switch`. The knob moves ease-in-out and stretches 12% on the way; Motion runs it
// so the knob only moves when `on` changes, never when the switch first appears.
const knob = 20;
const travel = 18;
const stretch = [knob, knob * 1.12, knob];

export function Switch({
	on,
	label,
	onToggle,
}: {
	on: boolean;
	label: string;
	onToggle?: () => void;
}) {
	return (
		<button
			type="button"
			role="switch"
			aria-checked={on}
			aria-label={label}
			className={cx("switch", on && "on")}
			onClick={onToggle}
		>
			<m.span
				className="knob"
				initial={false}
				animate={on ? "on" : "off"}
				variants={{
					on: { x: travel, width: stretch },
					off: { x: 0, width: stretch },
				}}
				transition={move}
			/>
		</button>
	);
}
