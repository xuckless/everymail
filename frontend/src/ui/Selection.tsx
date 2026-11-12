import { m } from "motion/react";
import { useId } from "react";
import { cx } from "./cx";
import { useTiming } from "./motion";

// The highlight behind the selected row of a list, which glides ease-in-out to the next selected
// row: every row of one list passes the same `group` (from useSelectionGroup), and only the row
// the view marks selected renders <SelectionMark>. Motion moves it by its shared layoutId, so no
// list carries row heights. The row needs the `selectable` class.
export const useSelectionGroup = useId;

export function SelectionMark({
	group,
	className = "strong",
}: {
	group: string;
	// `strong` on a panel, nothing on the page's own background.
	className?: string;
}) {
	const { move } = useTiming();
	return (
		<m.span
			layoutId={group}
			className={cx("sel-bar selection-mark", className)}
			transition={move}
			aria-hidden="true"
		/>
	);
}
