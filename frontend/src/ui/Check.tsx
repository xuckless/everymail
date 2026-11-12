import { cx } from "./cx";
import { Icon } from "./Icon";

// The tick on a chosen menu item, crossfading in and out with the choice.
export function Check({ on }: { on: boolean }) {
	return (
		<span className="swap">
			<Icon name="check" size="sm" className={cx("accent", !on && "hide")} />
		</span>
	);
}
