import { type ReactNode, useEffect } from "react";
import { unset, useSend } from "../engine";
import { Sidebar as SidebarState } from "../gen/ui/v1/common_pb";
import type { ShellView } from "../gen/ui/v1/shell_pb";
import { useLayout } from "../presentation";
import { cx } from "../ui/cx";
import { ComposeWindow } from "./ComposeWindow";
import { ConnectionStatus } from "./ConnectionStatus";
import { NoticeBar } from "./NoticeBar";
import { newMail } from "./places";
import { Sidebar, SidebarToggle } from "./Sidebar";
import { TabBar } from "./TabBar";

// The responsive frame around every screen, chosen per screen in Screen.tsx:
//   desktop: the sidebar, then the screen's panels in `.frame-main`. The engine's `sidebar`
//   collapses it: it slides away ease-in-out, the main area takes the width, and a narrow gutter
//   keeps its toggle and the connection dot;
//   phone (below 820px): the screen full height, with the tab bar on `tabs` screens.
// `chrome: "none"` is sign-in and connect, which show no navigation. Every frame carries the notice
// bar and the composer, and ⌘N or Ctrl+N opens a new mail wherever the navigation shows. The frame
// carries `data-layout` (phone, list, split) so screen CSS can follow the layout.
export type Chrome =
	| { chrome: "none" }
	| { chrome: "sidebar"; phone: "tabs" | "bare" };

export function Frame({
	view,
	chrome,
	children,
}: {
	view: ShellView;
	chrome: Chrome;
	children: ReactNode;
}) {
	const layout = useLayout();
	const { connection } = view;
	const overlays = (
		<>
			<NoticeBar notice={view.notice} />
			<ComposeWindow composer={view.composer} />
		</>
	);
	if (chrome.chrome === "none")
		return (
			<div className="frame frame-bare" data-layout={layout}>
				<main className="frame-screen">{children}</main>
				<ConnectionStatus
					connection={connection}
					form="dot"
					className="corner-status"
				/>
				{overlays}
			</div>
		);
	const nav = view.navigation ?? unset("ShellView.navigation");
	if (layout === "phone")
		return (
			<div className="frame frame-phone" data-layout={layout}>
				<NewMailKey />
				<main className="frame-screen">{children}</main>
				{chrome.phone === "tabs" && <TabBar nav={nav} />}
				<ConnectionStatus
					connection={connection}
					form="dot"
					className="corner-status"
				/>
				{overlays}
			</div>
		);
	const collapsed = sidebarCollapsed(view.sidebar);
	return (
		<div
			className={cx("frame", collapsed && "frame-collapsed")}
			data-layout={layout}
		>
			<NewMailKey />
			<Sidebar nav={nav} connection={connection} hidden={collapsed} />
			{collapsed && (
				<div className="gutter">
					<SidebarToggle collapsed />
					<ConnectionStatus connection={connection} form="dot" />
				</div>
			)}
			<main className="frame-main">{children}</main>
			{overlays}
		</div>
	);
}

// The new mail shortcut the engine writes (Navigation.new_mail_shortcut): ⌘N or Ctrl+N, whichever
// the platform uses; either modifier is accepted.
function NewMailKey() {
	const send = useSend();
	useEffect(() => {
		const key = (event: KeyboardEvent) => {
			if (event.key.toLowerCase() === "n" && (event.metaKey || event.ctrlKey)) {
				event.preventDefault();
				send(newMail);
			}
		};
		document.addEventListener("keydown", key);
		return () => document.removeEventListener("keydown", key);
	}, [send]);
	return null;
}

function sidebarCollapsed(sidebar: SidebarState) {
	switch (sidebar) {
		case SidebarState.EXPANDED:
			return false;
		case SidebarState.COLLAPSED:
			return true;
		case SidebarState.UNSPECIFIED:
			return unset("Sidebar");
	}
}
