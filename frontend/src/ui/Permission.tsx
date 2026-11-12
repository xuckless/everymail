import { unset } from "../engine";
import { type Permission, PermissionLevel } from "../gen/ui/v1/common_pb";
import { useLayout } from "../presentation";
import { cx } from "./cx";
import { SettingLine } from "./Lines";
import { Segmented } from "./Segmented";

// One action an agent may take, with its Off, Ask and Auto control: on an agent, in the builder
// and in the settings' defaults. Choosing a level has no intent yet, so the control is drawn
// without one. When Auto is not offered the contract's reason replaces the hint, in the warning
// colour, and the control offers Off and Ask only.
const levels = [
	{ level: PermissionLevel.OFF, label: "Off" },
	{ level: PermissionLevel.ASK, label: "Ask" },
	{ level: PermissionLevel.AUTO, label: "Auto" },
];

function offer({ levels: offered, hint }: Permission) {
	switch (offered.case) {
		case "all":
			return { choices: levels, note: hint, noteClass: "subtle" };
		case "askAtMost":
			return {
				choices: levels.slice(0, 2),
				note: offered.value.reason,
				noteClass: "perm-warn",
			};
		case undefined:
			return unset("Permission.levels");
	}
}

export function PermissionLine({ permission }: { permission: Permission }) {
	const touch = useLayout() === "phone";
	const { choices, note, noteClass } = offer(permission);
	return (
		<SettingLine label={permission.action} hint={note} hintClass={noteClass}>
			<Segmented
				kind="radio"
				label={permission.action}
				touch={touch}
				className={cx("perm-seg", choices.length === 2 && "two")}
				segments={choices.map(({ level, label }) => ({
					key: label,
					label,
					selected: level === permission.level,
				}))}
			/>
		</SettingLine>
	);
}
