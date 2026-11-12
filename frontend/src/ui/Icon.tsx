import icons from "@everymail/design/icons.svg?url";
import { cx } from "./cx";

// The ids of design/icons.svg.
export type IconName =
	| "inbox"
	| "calendar"
	| "briefing"
	| "clock"
	| "settings"
	| "check"
	| "plus"
	| "chevron-left"
	| "chevron-right"
	| "chevron-down"
	| "compose"
	| "archive"
	| "star"
	| "send"
	| "receipt"
	| "quarantine"
	| "shield"
	| "close"
	| "play"
	| "retry"
	| "search"
	| "summary"
	| "lock"
	| "snooze"
	| "link-off"
	| "arrow-right"
	| "forward"
	| "reply"
	| "reply-all"
	| "trash"
	| "folder"
	| "unread"
	| "sidebar"
	| "approvals"
	| "file"
	| "file-text"
	| "file-zip"
	| "image"
	| "table"
	| "presentation";

// Decorative; the control around it carries the label.
export function Icon({
	name,
	size,
	className,
}: {
	name: IconName;
	size?: "sm" | "lg";
	className?: string;
}) {
	return (
		<svg className={cx("i", size, className)} aria-hidden="true">
			<use href={`${icons}#${name}`} />
		</svg>
	);
}
