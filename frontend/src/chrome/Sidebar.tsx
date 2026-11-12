import type { ReactNode } from "react";
import { unset, useSend } from "../engine";
import { Form, Sidebar as SidebarState } from "../gen/ui/v1/common_pb";
import {
	type ConnectionView,
	Destination,
	type NavAgent,
	type Navigation,
	type NavTrash,
} from "../gen/ui/v1/shell_pb";
import { useDraft } from "../presentation";
import { Button, IconButton } from "../ui/Button";
import { cx } from "../ui/cx";
import { agentTone, Dot } from "../ui/Dot";
import { useDisclosure } from "../ui/disclosure";
import { Field } from "../ui/Field";
import { Icon } from "../ui/Icon";
import { Logo } from "../ui/Logo";
import { OutcomeNote, useOutcome } from "../ui/Outcome";
import { ConnectionStatus } from "./ConnectionStatus";
import { DraftRow } from "./DraftRow";
import {
	account,
	allInboxes,
	folder,
	folderIcon,
	folderIntent,
	newMail,
	settings,
} from "./places";

// The desktop sidebar on every screen: the brand and its collapse toggle, Compose, the mailboxes,
// calendar, briefing and approvals, the folders, the agents (the agent list: a click opens the
// agent's page), and the user with settings. A thin line (`.side-sep`) separates the groups because
// the owner asked for one. Saved drafts and Trash show when there are any.
export function Sidebar({
	nav,
	connection,
	hidden,
}: {
	nav: Navigation;
	connection: ConnectionView | undefined;
	hidden: boolean;
}) {
	// Required parts of the navigation: a missing one is a decode error.
	const allInboxesNav = nav.allInboxes ?? unset("Navigation.all_inboxes");
	const calendarNav = nav.calendar ?? unset("Navigation.calendar");
	const briefingNav = nav.briefing ?? unset("Navigation.briefing");
	const approvalsNav = nav.approvals ?? unset("Navigation.approvals");
	const send = useSend();
	return (
		<aside className="sidebar" aria-label="Sidebar" inert={hidden}>
			<div className="sidebar-inner">
				<div className="sidebar-head">
					<Logo brand={{ kind: "mark" }} size={22} />
					<span className="wordmark grow">EveryMail</span>
					<SidebarToggle collapsed={false} />
				</div>
				<Button
					variant="primary"
					className="compose"
					onClick={() => send(newMail)}
				>
					<Icon name="compose" />
					<span className="grow">Compose</span>
					<span className="kbd">{nav.newMailShortcut}</span>
				</Button>
				<nav className="sidebar-list scroll noscroll" aria-label="Main">
					<NavRow
						current={allInboxesNav.current}
						onClick={() => send(allInboxes)}
						lead={<Icon name="inbox" />}
						label="All inboxes"
						note={allInboxesNav.unread}
					/>
					{nav.accounts.map((each) => (
						<NavRow
							key={each.id}
							current={each.current}
							onClick={() => send(account(each.id))}
							lead={<Logo brand={{ kind: "mail", provider: each.provider }} />}
							label={each.name}
							note={each.unread}
						/>
					))}
					<div className="side-sep" />
					<NavRow
						current={calendarNav.current}
						onClick={() => send({ case: "openCalendar", value: {} })}
						lead={<Icon name="calendar" />}
						label="Calendar"
						note={calendarNav.nextEvent}
					/>
					<NavRow
						current={briefingNav.current}
						onClick={() => send({ case: "openBriefing", value: {} })}
						lead={<Icon name="briefing" />}
						label="Briefing"
						note={
							briefingNav.unread && (
								<Dot tone="unread" label="Unread" className="badge-dot" />
							)
						}
					/>
					<NavRow
						current={approvalsNav.current}
						onClick={() => send({ case: "openApprovals", value: {} })}
						lead={<Icon name="approvals" />}
						label={approvalsNav.label}
						note={approvalsNav.waiting}
					/>
					<div className="side-sep" />
					<NewFolder nav={nav} />
					{nav.folders.map((folder) => (
						<NavRow
							key={folder.id}
							current={folder.current}
							onClick={() => send(folderIntent(folder))}
							lead={<Icon name={folderIcon(folder)} />}
							label={folder.name}
							note={folder.count}
						/>
					))}
					{nav.trash && <Trash trash={nav.trash} />}
					{nav.drafts.length > 0 && (
						<>
							<div className="side-sep" />
							<div className="group-label">
								<span className="label grow">Drafts</span>
							</div>
							{nav.drafts.map((draft) => (
								<DraftRow
									key={draft.draftId}
									draft={draft}
									className="menu-row press nav-row nav-draft"
									lead={<Icon name="compose" />}
									onPick={send}
								/>
							))}
						</>
					)}
					<div className="side-sep" />
					<GroupLabel
						label="Agents"
						add="New agent"
						onAdd={() => send({ case: "openAgentBuilder", value: {} })}
					/>
					{nav.systemAgents.map((agent) => (
						<AgentRow key={agent.id} agent={agent} />
					))}
					<span className="group-gap" />
					{nav.userAgents.map((agent) => (
						<AgentRow key={agent.id} agent={agent} />
					))}
				</nav>
				<ConnectionStatus connection={connection} form="line" />
				<div className="sidebar-user">
					<span className="grow user-name trunc">{nav.userName}</span>
					<IconButton
						icon="settings"
						label="Settings"
						aria-current={
							nav.destination === Destination.SETTINGS ? "page" : undefined
						}
						onClick={() => send(settings)}
					/>
				</div>
			</div>
		</aside>
	);
}

function NavRow({
	current,
	onClick,
	lead,
	label,
	note,
}: {
	current: boolean | undefined;
	onClick: () => void;
	lead: ReactNode;
	label: string;
	note: ReactNode;
}) {
	return (
		<button
			type="button"
			className="menu-row press nav-row"
			aria-current={current ? "page" : undefined}
			onClick={onClick}
		>
			{lead}
			<span className="grow trunc">{label}</span>
			<span className="xs subtle">{note}</span>
		</button>
	);
}

function Trash({ trash }: { trash: NavTrash }) {
	const send = useSend();
	return (
		<NavRow
			current={trash.current}
			onClick={() => send(folder(trash.folderId))}
			lead={<Icon name="trash" />}
			label={trash.name}
			note={trash.count}
		/>
	);
}

// The "+" by Folders opens a field under the label (Navigation.new_folder); Enter creates the
// folder. A refused name comes back as the form's outcome and shows under the field; the engine
// closes the field when the folder is created, and clears its text by a new revision.
function NewFolder({ nav }: { nav: Navigation }) {
	const { isOpen: open, toggle } = useDisclosure({
		case: "form",
		value: Form.NEW_FOLDER,
	}).of(nav.newFolder, "Navigation.new_folder");
	const outcome = useOutcome({ case: "form", value: Form.NEW_FOLDER });
	return (
		<>
			<GroupLabel label="Folders" add="New folder" open={open} onAdd={toggle} />
			{open && <FolderName key={nav.newFolderRevision} />}
			<OutcomeNote outcome={outcome} className="new-folder-note" />
		</>
	);
}

function FolderName() {
	const send = useSend();
	const [name, setName] = useDraft();
	return (
		<span className="reveal new-folder">
			<Field
				label="Folder name"
				hideLabel
				placeholder="Folder name"
				value={name}
				onChange={setName}
				onEnter={() => send({ case: "createFolder", value: { name } })}
				autoFocus
			/>
		</span>
	);
}

function GroupLabel({
	label,
	add,
	open,
	onAdd,
}: {
	label: string;
	add: string;
	open?: boolean;
	onAdd: () => void;
}) {
	return (
		<div className="group-label">
			<span className="label grow">{label}</span>
			<IconButton
				icon="plus"
				size="sm"
				label={add}
				aria-expanded={open}
				className={cx("chev", open && "open-plus")}
				onClick={onAdd}
			/>
		</div>
	);
}

function AgentRow({ agent }: { agent: NavAgent }) {
	const send = useSend();
	return (
		<NavRow
			current={agent.current}
			onClick={() =>
				send({ case: "selectAgent", value: { agentId: agent.id } })
			}
			lead={<Dot tone={agentTone(agent.status)} className="agent-dot" />}
			label={agent.name}
			note={agent.note}
		/>
	);
}

// Collapses the sidebar from its head, and expands it from the corner it leaves.
export function SidebarToggle({ collapsed }: { collapsed: boolean }) {
	const send = useSend();
	return (
		<IconButton
			icon="sidebar"
			label={collapsed ? "Show sidebar" : "Hide sidebar"}
			aria-expanded={!collapsed}
			onClick={() =>
				send({
					case: "setSidebar",
					value: {
						sidebar: collapsed ? SidebarState.EXPANDED : SidebarState.COLLAPSED,
					},
				})
			}
		/>
	);
}
