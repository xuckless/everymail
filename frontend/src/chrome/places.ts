// Where the chrome's entries lead: the one table of destinations shared by the rail and the
// phone's tab bar, and the intents of mailboxes and folders shared by the sidebar and the phone's
// mailbox menu.

import { type IntentKind, unset } from "../engine";
import { SettingsSection } from "../gen/ui/v1/settings_pb";
import { Destination, type NavFolder } from "../gen/ui/v1/shell_pb";
import type { IconName } from "../ui/Icon";

export type Place = {
	destination: Destination;
	label: string;
	icon: IconName;
	intent: IntentKind;
};

export const allInboxes: IntentKind = {
	case: "openInbox",
	value: { scope: { scope: { case: "all", value: {} } } },
};

export const account = (id: string): IntentKind => ({
	case: "openInbox",
	value: { scope: { scope: { case: "accountId", value: id } } },
});

export const settings: IntentKind = {
	case: "openSettings",
	value: { section: SettingsSection.ACCOUNTS },
};

// Where every "AI is off" state leads: the Models settings section, to add a key.
export const modelSettings: IntentKind = {
	case: "openSettings",
	value: { section: SettingsSection.MODELS },
};

// In rail order; the tab bar shows the first four.
export const places: Place[] = [
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
	{
		destination: Destination.SETTINGS,
		label: "Settings",
		icon: "settings",
		intent: settings,
	},
];

export function folderIntent(folder: NavFolder): IntentKind {
	switch (folder.kind.case) {
		case "quarantine":
			return { case: "openQuarantine", value: {} };
		case "starred":
		case "sent":
		case "archive":
		case "user":
			return {
				case: "openInbox",
				value: { scope: { scope: { case: "folderId", value: folder.id } } },
			};
		case undefined:
			return unset("NavFolder.kind");
	}
}

// The design has no generic folder icon; a user folder shows the receipt drawn for Receipts.
export function folderIcon(folder: NavFolder): IconName {
	switch (folder.kind.case) {
		case "starred":
			return "star";
		case "sent":
			return "send";
		case "archive":
			return "archive";
		case "quarantine":
			return "quarantine";
		case "user":
			return "receipt";
		case undefined:
			return unset("NavFolder.kind");
	}
}
