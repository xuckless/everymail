import { type IntentKind, unset, useSend } from "../engine";
import { Destination, type Navigation } from "../gen/ui/v1/shell_pb";
import { cx } from "../ui/cx";
import { Dot } from "../ui/Dot";
import { Icon, type IconName } from "../ui/Icon";
import { SelectionMark, useSelectionGroup } from "../ui/Selection";
import { allInboxes } from "./places";

// The phone's bottom navigation: Inbox, Calendar, Briefing and Agents, with one highlight that
// moves ease-in-out to the current destination. Settings has no tab in the design.
const tabs: {
	destination: Destination;
	label: string;
	icon: IconName;
	intent: IntentKind;
}[] = [
	{
		destination: Destination.INBOX,
		label: "Inbox",
		icon: "inbox",
		intent: allInboxes,
	},
	{
		destination: Destination.CALENDAR,
		label: "Calendar",
		icon: "calendar",
		intent: { case: "openCalendar", value: {} },
	},
	{
		destination: Destination.BRIEFING,
		label: "Briefing",
		icon: "briefing",
		intent: { case: "openBriefing", value: {} },
	},
	{
		destination: Destination.AGENTS,
		label: "Agents",
		icon: "clock",
		intent: { case: "openAgents", value: {} },
	},
];

export function TabBar({ nav }: { nav: Navigation }) {
	const send = useSend();
	const group = useSelectionGroup();
	const badge = (destination: Destination) => {
		switch (destination) {
			case Destination.BRIEFING:
				return (
					(nav.briefing ?? unset("Navigation.briefing")).unread && (
						<Dot tone="unread" label="Unread" />
					)
				);
			case Destination.AGENTS:
				return (
					(nav.approvals ?? unset("Navigation.approvals")).waiting && (
						<Dot tone="wait" label="Waiting" />
					)
				);
			case Destination.NONE:
			case Destination.INBOX:
			case Destination.CALENDAR:
			case Destination.SETTINGS:
				return null;
			case Destination.UNSPECIFIED:
				return unset("Destination");
		}
	};
	return (
		<nav className="tabbar" aria-label="Main">
			<div
				className="tabbar-tabs"
				style={{
					gridTemplateColumns: `repeat(${tabs.length}, minmax(0, 1fr))`,
				}}
			>
				{tabs.map((tab) => (
					<button
						key={tab.destination}
						type="button"
						className={cx(
							"tab press selectable",
							tab.destination === nav.destination && "on",
						)}
						aria-current={
							tab.destination === nav.destination ? "page" : undefined
						}
						onClick={() => send(tab.intent)}
					>
						{tab.destination === nav.destination && (
							<SelectionMark group={group} />
						)}
						<span className="tab-icon">
							<Icon name={tab.icon} size="lg" />
							{badge(tab.destination)}
						</span>
						{tab.label}
					</button>
				))}
			</div>
		</nav>
	);
}
