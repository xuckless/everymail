import { useSend } from "../engine";
import type { Navigation } from "../gen/ui/v1/shell_pb";
import { cx } from "../ui/cx";
import { Icon } from "../ui/Icon";
import { Logo } from "../ui/Logo";
import { Popover } from "../ui/Overlay";
import { stagger } from "../ui/stagger";
import { account, allInboxes, folderIntent, settings } from "./places";

// The phone's mailbox switcher: the screen title opens a menu of all inboxes, the accounts, the
// folders, Add account and Settings (the tab bar has no Settings tab).
export function MailboxMenu({
	nav,
	title,
}: {
	nav: Navigation;
	title: string;
}) {
	const send = useSend();
	return (
		<Popover
			label="Mailboxes"
			className="mailbox-menu"
			trigger={(trigger) => (
				<button type="button" className="h1 press mailbox-title" {...trigger}>
					{title}
					<Icon
						name="chevron-right"
						className={cx("subtle chev", trigger["aria-expanded"] && "open")}
					/>
				</button>
			)}
		>
			{(dismiss) => {
				const go = (intent: Parameters<typeof send>[0]) => () => {
					dismiss();
					send(intent);
				};
				const folders = 1 + nav.accounts.length;
				return (
					<>
						<button
							type="button"
							role="menuitem"
							className="menu-row touch row-in stagger"
							style={stagger(0)}
							onClick={go(allInboxes)}
						>
							<span className="grow">All inboxes</span>
							<span className="xs subtle">{nav.allInboxes?.unread}</span>
						</button>
						{nav.accounts.map((each, i) => (
							<button
								key={each.id}
								type="button"
								role="menuitem"
								className="menu-row touch row-in stagger"
								style={stagger(1 + i)}
								onClick={go(account(each.id))}
							>
								<Logo
									brand={{ kind: "mail", provider: each.provider }}
									size={22}
								/>
								<span className="grow">{each.name}</span>
								<span className="xs subtle">{each.unread}</span>
							</button>
						))}
						<div className="side-sep" aria-hidden="true" />
						{nav.folders.map((folder, i) => (
							<button
								key={folder.id}
								type="button"
								role="menuitem"
								className="menu-row touch row-in stagger"
								style={stagger(folders + i)}
								onClick={go(folderIntent(folder))}
							>
								<span className="grow">{folder.name}</span>
								<span className="xs subtle">{folder.count}</span>
							</button>
						))}
						<div className="side-sep" aria-hidden="true" />
						<button
							type="button"
							role="menuitem"
							className="menu-row touch row-in stagger"
							style={stagger(folders + nav.folders.length)}
							onClick={go({ case: "openConnect", value: {} })}
						>
							<span className="grow">Add account</span>
						</button>
						<button
							type="button"
							role="menuitem"
							className="menu-row touch row-in stagger"
							style={stagger(folders + nav.folders.length + 1)}
							onClick={go(settings)}
						>
							<span className="grow">Settings</span>
						</button>
					</>
				);
			}}
		</Popover>
	);
}
