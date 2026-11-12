import { useSend } from "../engine";
import type {
	ConnectionView,
	NavAgent,
	Navigation,
} from "../gen/ui/v1/shell_pb";
import { Button, IconButton } from "../ui/Button";
import { agentTone, Dot } from "../ui/Dot";
import { Icon } from "../ui/Icon";
import { Logo } from "../ui/Logo";
import { ConnectionStatus } from "./ConnectionStatus";
import {
	account,
	allInboxes,
	folderIcon,
	folderIntent,
	settings,
} from "./places";

// The desktop sidebar of the inbox, calendar, Quarantine and briefing screens: mailboxes,
// calendar and briefing, folders, agents, and the user with settings. A thin line (`.side-sep`)
// separates the groups because the owner asked for one.
export function Sidebar({
	nav,
	connection,
}: {
	nav: Navigation;
	connection: ConnectionView | undefined;
}) {
	const send = useSend();
	return (
		<aside className="sidebar" aria-label="Mailboxes">
			<div className="sidebar-head">
				<Logo brand={{ kind: "mark" }} size={18} />
			</div>
			{/* Compose has no intent yet. */}
			<Button variant="primary" className="compose">
				<Icon name="compose" />
				<span className="grow">Compose</span>
				<span className="kbd">⌘N</span>
			</Button>
			<nav className="sidebar-list scroll noscroll" aria-label="Main">
				<button
					type="button"
					className="menu-row press nav-row"
					aria-current={nav.allInboxes?.current ? "page" : undefined}
					onClick={() => send(allInboxes)}
				>
					<Icon name="inbox" />
					<span className="grow">All inboxes</span>
					<span className="xs subtle">{nav.allInboxes?.unread}</span>
				</button>
				{nav.accounts.map((each) => (
					<button
						key={each.id}
						type="button"
						className="menu-row press nav-row"
						aria-current={each.current ? "page" : undefined}
						onClick={() => send(account(each.id))}
					>
						<Logo brand={{ kind: "mail", provider: each.provider }} />
						<span className="grow trunc">{each.name}</span>
						<span className="xs subtle">{each.unread}</span>
					</button>
				))}
				<div className="side-sep" />
				<button
					type="button"
					className="menu-row press nav-row"
					aria-current={nav.calendar?.current ? "page" : undefined}
					onClick={() => send({ case: "openCalendar", value: {} })}
				>
					<Icon name="calendar" />
					<span className="grow">Calendar</span>
					<span className="xs subtle">{nav.calendar?.nextEvent}</span>
				</button>
				<button
					type="button"
					className="menu-row press nav-row"
					aria-current={nav.briefing?.current ? "page" : undefined}
					onClick={() => send({ case: "openBriefing", value: {} })}
				>
					<Icon name="briefing" />
					<span className="grow">Briefing</span>
					{nav.briefing?.unread && (
						<Dot tone="unread" label="Unread" className="badge-dot" />
					)}
				</button>
				<div className="side-sep" />
				<div className="label group-label">Folders</div>
				{nav.folders.map((folder) => (
					<button
						key={folder.id}
						type="button"
						className="menu-row press nav-row"
						aria-current={folder.current ? "page" : undefined}
						onClick={() => send(folderIntent(folder))}
					>
						<Icon name={folderIcon(folder)} />
						<span className="grow trunc">{folder.name}</span>
						<span className="xs subtle">{folder.count}</span>
					</button>
				))}
				<div className="side-sep" />
				<div className="label group-label">Agents</div>
				{nav.systemAgents.map((agent) => (
					<AgentRow key={agent.id} agent={agent} />
				))}
				<span className="group-gap" />
				{nav.userAgents.map((agent) => (
					<AgentRow key={agent.id} agent={agent} />
				))}
				<button
					type="button"
					className="menu-row press nav-row subtle"
					onClick={() => send({ case: "openAgentBuilder", value: {} })}
				>
					<Icon name="plus" />
					<span>New agent</span>
				</button>
			</nav>
			<ConnectionStatus connection={connection} form="line" />
			<div className="sidebar-user">
				<span className="grow user-name trunc">{nav.userName}</span>
				<IconButton
					icon="settings"
					label="Settings"
					onClick={() => send(settings)}
				/>
			</div>
		</aside>
	);
}

function AgentRow({ agent }: { agent: NavAgent }) {
	const send = useSend();
	return (
		<button
			type="button"
			className="menu-row press nav-row"
			onClick={() =>
				send({ case: "selectAgent", value: { agentId: agent.id } })
			}
		>
			<Dot tone={agentTone(agent.status)} className="agent-dot" />
			<span className="grow trunc">{agent.name}</span>
			<span className="xs subtle">{agent.note}</span>
		</button>
	);
}
