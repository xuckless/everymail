import type { ReactNode } from "react";
import type { ConnectionView, Navigation } from "../gen/ui/v1/shell_pb";
import { useLayout } from "../presentation";
import { cx } from "../ui/cx";
import { ConnectionStatus } from "./ConnectionStatus";
import { Rail } from "./Rail";
import { Sidebar } from "./Sidebar";
import { TabBar } from "./TabBar";

// The responsive frame around every screen, chosen per screen in Screen.tsx:
//   desktop: the sidebar or the rail, then the screen's panels in `.frame-main`;
//   phone (below 820px): the screen full height, with the tab bar on `tabs` screens.
// `chrome: "none"` is sign-in and connect, which show no navigation. The frame carries
// `data-layout` (phone, list, split) so screen CSS can follow the layout without media queries.
export type Chrome =
	| { chrome: "none" }
	| { chrome: "sidebar" | "rail"; nav: Navigation; phone: "tabs" | "bare" };

export function Frame({
	connection,
	children,
	...chrome
}: Chrome & { connection: ConnectionView | undefined; children: ReactNode }) {
	const layout = useLayout();
	const corner = (
		<ConnectionStatus
			connection={connection}
			form="dot"
			className="corner-status"
		/>
	);
	if (layout === "phone" || chrome.chrome === "none")
		return (
			<div
				className={cx(
					"frame",
					layout === "phone" ? "frame-phone" : "frame-bare",
				)}
				data-layout={layout}
			>
				<main className="frame-screen">{children}</main>
				{layout === "phone" &&
					chrome.chrome !== "none" &&
					chrome.phone === "tabs" && <TabBar nav={chrome.nav} />}
				{corner}
			</div>
		);
	return (
		<div className="frame" data-layout={layout}>
			{chrome.chrome === "sidebar" ? (
				<Sidebar nav={chrome.nav} connection={connection} />
			) : (
				<Rail nav={chrome.nav} connection={connection} />
			)}
			<main className="frame-main">{children}</main>
		</div>
	);
}
