import type { ReactNode } from "react";

// The phone's one screen head (`.phone-head`): under the status corner, a bar (back or close, the
// title or a toolbar, actions), then whatever sits under it (search, tabs, progress, a caption).
export function PhoneHead({
	bar,
	children,
}: {
	bar: ReactNode;
	children?: ReactNode;
}) {
	return (
		<header className="phone-head">
			<div className="phone-bar">{bar}</div>
			{children}
		</header>
	);
}
