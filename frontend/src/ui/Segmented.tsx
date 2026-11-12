import { m } from "motion/react";
import type { CSSProperties, MouseEvent, ReactNode } from "react";
import { cx } from "./cx";
import { useTiming } from "./motion";

// The indicator stretches 12% on its way, as the switch's knob does (motion rule 5); with no
// segment selected it is hidden.
const moving = { scaleX: [1, 1.12, 1], opacity: 1 };
const none = { "-1": { opacity: 0 } };

export type Segment = {
	key: string;
	label: string;
	// As the view marks it; at most one.
	selected: boolean;
	// The press, for a control that starts something where it was pressed (the theme's reveal).
	onSelect?: (event: MouseEvent<HTMLButtonElement>) => void;
	// Drawn before the label, such as a calendar's colour dot.
	lead?: ReactNode;
};

// The design's `.seg`: one indicator that moves ease-in-out to the selected option. `tabs` switches what is shown (role tablist); `radio` is a choice (radiogroup).
export function Segmented({
	label,
	kind,
	segments,
	touch,
	className,
}: {
	label: string;
	kind: "tabs" | "radio";
	segments: Segment[];
	touch?: boolean;
	className?: string;
}) {
	const { move } = useTiming();
	const at = segments.findIndex((segment) => segment.selected);
	const classes = cx("seg", touch && "touch", className);
	const indicator = (
		<m.span
			className="ind"
			style={{ "--n": segments.length } as CSSProperties}
			initial={false}
			animate={String(at)}
			variants={{
				...none,
				...Object.fromEntries(
					segments.map((_, i) => [i, { x: `${i * 100}%`, ...moving }]),
				),
			}}
			transition={move}
		/>
	);
	const option = (segment: Segment) => ({
		type: "button" as const,
		className: cx("opt", segment.selected && "on"),
		onClick: segment.onSelect,
		children: (
			<>
				{segment.lead}
				{segment.label}
			</>
		),
	});
	switch (kind) {
		case "tabs":
			return (
				<div className={classes} role="tablist" aria-label={label}>
					{indicator}
					{segments.map((segment) => (
						<button
							{...option(segment)}
							key={segment.key}
							role="tab"
							aria-selected={segment.selected}
						/>
					))}
				</div>
			);
		case "radio":
			return (
				<div className={classes} role="radiogroup" aria-label={label}>
					{indicator}
					{segments.map((segment) => (
						// biome-ignore lint/a11y/useSemanticElements: as drawn, a segment is a button that moves the indicator
						<button
							{...option(segment)}
							key={segment.key}
							role="radio"
							aria-checked={segment.selected}
						/>
					))}
				</div>
			);
	}
}
