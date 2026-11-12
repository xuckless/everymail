import "./agents.css";
import { unset, useSend } from "../engine";
import {
	type AgentChat,
	type AgentDetail,
	AgentDetailPane,
	type AgentRow,
	AgentsPane,
	type AgentsView,
} from "../gen/ui/v1/agents_pb";
import { useLayout } from "../presentation";
import { Button, IconButton } from "../ui/Button";
import { cx } from "../ui/cx";
import { agentTone, Dot } from "../ui/Dot";
import { Overlap } from "../ui/Overlap";
import { Row, Rows } from "../ui/Rows";
import { Segmented } from "../ui/Segmented";
import { SelectionMark, useSelectionGroup } from "../ui/Selection";
import { SelectList, useOptionIds } from "../ui/SelectList";
import { Switch } from "../ui/Switch";
import {
	agentChat,
	Chat,
	chatCoversPhone,
	chatOf,
	currentDetailPane,
	DetailPanes,
} from "./agents/Chat";
import {
	Activity,
	AgentActions,
	DetailHead,
	YourInstructions,
} from "./agents/Detail";
import { QueuePane, QueuePanel } from "./agents/Queue";
import { isActive } from "./agents/status";
import type { ScreenProps } from "./Screen";

// The agents, the selected one's detail and the approval queue. A wide layout shows all three; a
// narrower one shows the engine's current pane. The new agent button opens the builder. The active
// switch, Run now, Edit, Delete, adding a time and permission levels have no intent yet and are
// drawn without one.
export function Agents({ view }: ScreenProps<AgentsView>) {
	const layout = useLayout();
	switch (layout) {
		case "phone":
			return <PhoneAgents view={view} />;
		case "list":
			return <NarrowAgents view={view} />;
		case "split":
			return (
				<>
					<AgentList view={view} />
					<DetailPanel detail={view.detail} />
					<QueuePanel queue={view.queue} />
				</>
			);
	}
}

const openBuilder = { case: "openAgentBuilder", value: {} } as const;

function NewAgent({ touch }: { touch?: boolean }) {
	const send = useSend();
	return (
		<IconButton
			icon="plus"
			label="New agent"
			size={touch ? "lg" : undefined}
			touch={touch}
			className="new-agent"
			onClick={() => send(openBuilder)}
		/>
	);
}

function currentPane({ panes }: AgentsView) {
	return panes.find((tab) => tab.current)?.pane ?? unset("AgentsView.panes");
}

function PaneTabs({ view, touch }: { view: AgentsView; touch?: boolean }) {
	const send = useSend();
	return (
		<Segmented
			kind="tabs"
			label="View"
			touch={touch}
			segments={view.panes.map((tab) => ({
				key: String(tab.pane),
				label: tab.label,
				selected: tab.current,
				onSelect: () =>
					send({ case: "selectAgentsPane", value: { pane: tab.pane } }),
			}))}
		/>
	);
}

function useSelect() {
	const send = useSend();
	return (agentId: string) => () =>
		send({ case: "selectAgent", value: { agentId } });
}

function AgentList({ view }: { view: AgentsView }) {
	const select = useSelect();
	const group = useSelectionGroup();
	const optionId = useOptionIds();
	const { systemAgents, userAgents } = view;
	const selected = [...systemAgents, ...userAgents].find((row) => row.selected);
	const row = (agent: AgentRow, index: number) => (
		<Row
			key={agent.id}
			index={index}
			option={{ id: optionId(agent.id), selected: agent.selected }}
		>
			<button
				type="button"
				data-select
				className="press selectable list-agent"
				aria-current={agent.selected ? "true" : undefined}
				onClick={select(agent.id)}
			>
				{agent.selected && <SelectionMark group={group} />}
				<Dot tone={agentTone(agent.status)} />
				<span className="grow two-line">
					<span className="trunc strong">{agent.name}</span>
					<span className="trunc xs subtle">{agent.next}</span>
				</span>
			</button>
		</Row>
	);
	return (
		<section className="panel agent-list" aria-label="Agents">
			<div className="agent-list-head">
				<h1 className="h2 grow">Agents</h1>
				<NewAgent />
			</div>
			<SelectList
				label="Agents"
				active={selected && optionId(selected.id)}
				className="scroll noscroll agent-list-scroll"
			>
				<div className="label agent-group" aria-hidden="true">
					System
				</div>
				<Rows label="System agents" group>
					{systemAgents.map((agent, i) => row(agent, i))}
				</Rows>
				<div className="label agent-group second" aria-hidden="true">
					Yours
				</div>
				<Rows label="Your agents" group>
					{userAgents.map((agent, i) => row(agent, systemAgents.length + i))}
				</Rows>
			</SelectList>
		</section>
	);
}

function DetailPanel({ detail }: { detail: AgentsView["detail"] }) {
	return (
		<section className="panel agent-main" aria-label="Selected agent">
			<Overlap
				id={detail.case === "agent" ? detail.value.id : "none"}
				className="detail-stack"
			>
				<Detail detail={detail} />
			</Overlap>
		</section>
	);
}

function Detail({ detail }: { detail: AgentsView["detail"] }) {
	switch (detail.case) {
		case "agent":
			return <AgentPanes detail={detail.value} />;
		case "none":
			return null;
		case undefined:
			return unset("AgentsView.detail");
	}
}

// Without chat, the head scrolls with the activity as drawn. With chat, the head and the pane list
// stay on top and the current pane fills the rest, old and new overlapping.
function AgentPanes({ detail }: { detail: AgentDetail }) {
	const chat = agentChat(detail.chat);
	const activity = (
		<div className="scroll detail-scroll">
			<div className="agent-detail">
				{!chat && <DetailHead detail={detail} />}
				<Activity detail={detail} />
			</div>
		</div>
	);
	if (!chat) return activity;
	const pane = currentDetailPane(chat);
	return (
		<div className="agent-panes">
			<div className="agent-detail agent-panes-head">
				<DetailHead detail={detail} />
				<DetailPanes chat={chat} />
			</div>
			<Overlap id={String(pane)} className="detail-stack">
				{pane === AgentDetailPane.CHAT ? <Chat chat={chat} /> : activity}
			</Overlap>
		</div>
	);
}

function NarrowAgents({ view }: { view: AgentsView }) {
	return (
		<div className="agents-narrow">
			<div className="narrow-bar">
				<PaneTabs view={view} />
				<NewAgent />
			</div>
			<Overlap id={String(currentPane(view))} className="narrow-stack">
				<NarrowPane view={view} />
			</Overlap>
		</div>
	);
}

function NarrowPane({ view }: { view: AgentsView }) {
	const pane = currentPane(view);
	switch (pane) {
		case AgentsPane.AGENTS:
			return (
				<div className="narrow-pane">
					<AgentList view={view} />
					<DetailPanel detail={view.detail} />
				</div>
			);
		case AgentsPane.APPROVALS:
			return <QueuePanel queue={view.queue} />;
		case AgentsPane.UNSPECIFIED:
			return unset("AgentsPane");
	}
}

// The chat covers the whole screen while it is the selected agent's current pane; back returns to
// the agent's expanded row.
function PhoneAgents({ view }: { view: AgentsView }) {
	const { detail } = view;
	const chat = chatCoversPhone(view) ? chatOf(detail) : undefined;
	return (
		<Overlap id={chat ? "chat" : "agents"} className="agents-phone-stack">
			{chat && detail.case === "agent" ? (
				<PhoneChat name={detail.value.name} chat={chat} />
			) : (
				<PhoneList view={view} />
			)}
		</Overlap>
	);
}

function PhoneChat({ name, chat }: { name: string; chat: AgentChat }) {
	const send = useSend();
	return (
		<div className="agents-phone chat-phone">
			<header className="chat-phone-bar">
				<IconButton
					icon="chevron-left"
					label="Back"
					size="lg"
					touch
					onClick={() => send({ case: "back", value: {} })}
				/>
				<h1 className="h2 grow trunc">{name}</h1>
			</header>
			<Chat chat={chat} />
		</div>
	);
}

function PhoneList({ view }: { view: AgentsView }) {
	return (
		<div className="agents-phone">
			<header className="phone-head">
				<div className="phone-bar">
					<h1 className="h1 grow phone-title">Agents</h1>
					<NewAgent touch />
				</div>
				<PaneTabs view={view} touch />
			</header>
			<Overlap id={String(currentPane(view))} className="phone-stack">
				<PhonePane view={view} />
			</Overlap>
		</div>
	);
}

function PhonePane({ view }: { view: AgentsView }) {
	const pane = currentPane(view);
	switch (pane) {
		case AgentsPane.AGENTS:
			return (
				<div className="scroll noscroll phone-pane">
					<div className="label agent-group">System</div>
					<PhoneRows rows={view.systemAgents} first={0} detail={view.detail} />
					<div className="label agent-group">Yours</div>
					<PhoneRows
						rows={view.userAgents}
						first={view.systemAgents.length}
						detail={view.detail}
					/>
				</div>
			);
		case AgentsPane.APPROVALS:
			return <QueuePane queue={view.queue} />;
		case AgentsPane.UNSPECIFIED:
			return unset("AgentsPane");
	}
}

function PhoneRows({
	rows,
	first,
	detail,
}: {
	rows: AgentRow[];
	first: number;
	detail: AgentsView["detail"];
}) {
	return (
		<Rows label="Agents" className="phone-agents">
			{rows.map((row, i) => (
				<Row key={row.id} index={first + i} className="tile phone-agent">
					<PhoneAgent row={row} detail={detail} />
				</Row>
			))}
		</Rows>
	);
}

function PhoneAgent({
	row,
	detail,
}: {
	row: AgentRow;
	detail: AgentsView["detail"];
}) {
	const select = useSelect();
	return (
		<>
			<div className="phone-agent-head">
				<button
					type="button"
					className="press phone-agent-main"
					aria-expanded={row.selected}
					onClick={select(row.id)}
				>
					<Dot tone={agentTone(row.status)} />
					<span className="grow two-line">
						<span className="trunc strong">{row.name}</span>
						<span className="trunc xs subtle">{row.next}</span>
					</span>
				</button>
				<Switch on={isActive(row.status)} label={row.name} />
			</div>
			{row.selected && detail.case === "agent" && (
				<Expanded detail={detail.value} lastRun={row.lastRun} />
			)}
		</>
	);
}

function Expanded({
	detail,
	lastRun,
}: {
	detail: AgentDetail;
	lastRun: string;
}) {
	return (
		<div className={cx("reveal", "phone-expanded")}>
			<Instructions detail={detail} />
			<span className="settle xs subtle">Last run: {lastRun}</span>
			<span className="settle agent-actions-row">
				<OpenChat chat={agentChat(detail.chat)} />
				<AgentActions kind={detail.kind} />
			</span>
		</div>
	);
}

// On the phone, the chat pane is reached from the agent's expanded row.
function OpenChat({ chat }: { chat: AgentChat | undefined }) {
	const send = useSend();
	const tab = chat?.panes.find((each) => each.pane === AgentDetailPane.CHAT);
	if (!tab) return null;
	return (
		<Button
			variant="soft"
			onClick={() =>
				send({ case: "selectAgentDetailPane", value: { pane: tab.pane } })
			}
		>
			{tab.label}
		</Button>
	);
}

function Instructions({ detail }: { detail: AgentDetail }) {
	const { kind } = detail;
	switch (kind.case) {
		case "system":
			return (
				<>
					<span className="settle xs subtle">
						Built-in instructions, kept up to date by EveryMail
					</span>
					<span className="settle sm muted agent-text">
						{kind.value.builtInInstructions}
					</span>
					<YourInstructions
						system={kind.value}
						placeholder="Add your own instructions"
					/>
				</>
			);
		case "user":
			return (
				<span className="settle sm muted agent-text">
					{kind.value.instructions}
				</span>
			);
		case undefined:
			return unset("AgentDetail.kind");
	}
}
