import { unset } from "../engine";
import { type Permission, PermissionLevel } from "../gen/ui/v1/common_pb";
import { useLayout } from "../presentation";
import { cx } from "./cx";
import { SettingLine } from "./Lines";
import { Segmented } from "./Segmented";

// One action an agent may take, with its Off, Ask and Auto control: on an agent, in the builder
// and in the settings' defaults; the caller sends the chosen level. When Auto is not offered the
// contract's reason replaces the hint, in the warning colour, and the control offers Off and Ask.
const off = { level: PermissionLevel.OFF, label: "Off" };
const ask = { level: PermissionLevel.ASK, label: "Ask" };
const auto = { level: PermissionLevel.AUTO, label: "Auto" };

function offer({ levels: offered, hint }: Permission) {
	switch (offered.case) {
		case "all":
			return { choices: [off, ask, auto], note: hint, noteClass: "subtle" };
		case "askAtMost":
			return {
				choices: [off, ask],
				note: offered.value.reason,
				noteClass: "perm-warn",
			};
		case undefined:
			return unset("Permission.levels");
	}
}

export function PermissionLine({
	permission,
	onLevel,
}: {
	permission: Permission;
	onLevel: (level: PermissionLevel) => void;
}) {
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
					onSelect: () => onLevel(level),
				}))}
			/>
		</SettingLine>
	);
}
