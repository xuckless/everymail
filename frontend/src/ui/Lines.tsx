import type { ReactNode } from "react";
import { cx } from "./cx";
import { Switch } from "./Switch";

// A setting's line: what it is (with an optional hint under it) and its control on the right.
export function SettingLine({
	label,
	hint,
	hintClass = "subtle",
	children,
}: {
	label: ReactNode;
	hint?: string;
	hintClass?: string;
	children: ReactNode;
}) {
	return (
		<div className="setting-line">
			<span className="two-line grow setting-what">
				<span>{label}</span>
				{hint && <span className={cx("xs", hintClass)}>{hint}</span>}
			</span>
			{children}
		</div>
	);
}

// A line whose control is a switch named by the line's label.
export function SwitchLine({
	label,
	hint,
	on,
	onToggle,
}: {
	label: string;
	hint?: string;
	on: boolean;
	onToggle: () => void;
}) {
	return (
		<SettingLine label={label} hint={hint}>
			<Switch on={on} label={label} onToggle={onToggle} />
		</SettingLine>
	);
}
