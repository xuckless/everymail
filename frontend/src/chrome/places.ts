// Where the chrome's entries lead: the intents of mailboxes, folders and settings shared by the
// sidebar, the phone's tab bar and mailbox menu, and the screens that link to settings.

import { type IntentKind, unset } from "../engine";
import { SettingsSection } from "../gen/ui/v1/settings_pb";
import type { NavFolder } from "../gen/ui/v1/shell_pb";
import type { IconName } from "../ui/Icon";

export const allInboxes: IntentKind = {
	case: "openInbox",
	value: { scope: { scope: { case: "all", value: {} } } },
};

export const account = (id: string): IntentKind => ({
	case: "openInbox",
	value: { scope: { scope: { case: "accountId", value: id } } },
});

export const folder = (id: string): IntentKind => ({
	case: "openInbox",
	value: { scope: { scope: { case: "folderId", value: id } } },
});

// Compose, ⌘N and the phone's new-mail button.
export const newMail: IntentKind = {
	case: "openComposer",
	value: { kind: { case: "newMail", value: {} } },
};

export const settings: IntentKind = {
	case: "openSettings",
	value: { section: SettingsSection.ACCOUNTS },
};

// Where every "AI is off" state leads: the Models settings section, to add a key.
export const modelSettings: IntentKind = {
	case: "openSettings",
	value: { section: SettingsSection.MODELS },
};

export function folderIntent(each: NavFolder): IntentKind {
	switch (each.kind.case) {
		case "quarantine":
			return { case: "openQuarantine", value: {} };
		case "starred":
		case "sent":
		case "archive":
		case "user":
			return folder(each.id);
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
