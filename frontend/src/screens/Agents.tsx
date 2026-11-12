import "./agents.css";
import { unset, useSend } from "../engine";
import type {
	AgentList,
	AgentPage,
	AgentRow,
	AgentsView,
} from "../gen/ui/v1/agents_pb";
import { useLayout } from "../presentation";
import { Button, IconButton } from "../ui/Button";
import { agentTone, Dot } from "../ui/Dot";
import { Icon } from "../ui/Icon";
import { OutcomeNote, pulseOn, useOutcome } from "../ui/Outcome";
import { Overlap } from "../ui/Overlap";
import { Page } from "../ui/Page";
import { Row, Rows } from "../ui/Rows";
import { Segmented } from "../ui/Segmented";
import { Switch } from "../ui/Switch";
import { Activity } from "./agents/Activity";
import { Chat } from "./agents/Chat";
import { Configure } from "./agents/Configure";
import { Results } from "./agents/Results";
import { isActive, isRunning } from "./agents/status";
import type { ScreenProps } from "./Screen";

// One agent's page, or the list of agents (the phone's Agents tab; the desktop's sidebar is the
// list).
export function Agents({ view }: ScreenProps<AgentsView>) {
	const { page } = view;
	switch (page.case) {
		case "list":
			return <List list={page.value} />;
		case "agent":
			return <Agent page={page.value} />;
		case undefined:
			return unset("AgentsView.page");
	}
}

// The header (name, what it is, status, Run now and the active switch), the pane tabs and the
// current pane, old and new overlapping as the pane changes.
function Agent({ page }: { page: AgentPage }) {
	const send = useSend();
	const phone = useLayout() === "phone";
	return (
		<Page
			label={page.name}
			title={page.name}
			subtitle={page.subtitle}
			back
			fill
			className="page-wrap"
			actions={<Head page={page} />}
			tabs={
				<Segmented
					kind="tabs"
					label="Agent"
					touch={phone}
					className="agent-tabs"
					segments={page.panes.map((tab) => ({
						key: String(tab.pane),
						label: tab.label,
						selected: tab.current,
						onSelect: () =>
							send({
								case: "selectAgentDetailPane",
								value: { pane: tab.pane },
							}),
					}))}
				/>
			}
		>
			<Overlap id={`${page.id}-${page.pane.case}`} className="agent-stack">
				<Pane pane={page.pane} agentId={page.id} />
			</Overlap>
		</Page>
	);
}

// Status, Run now (its label crossfades to Running while the agent runs, and pulses when the run
// is done), and the active switch; a refusal (paused, deactivated, at a cap) shows under them.
function Head({ page }: { page: AgentPage }) {
	const send = useSend();
	const phone = useLayout() === "phone";
	const active = isActive(page.status);
	const agentId = page.id;
	const outcome = useOutcome({ case: "agentId", value: agentId });
	const running = isRunning(page.status);
	return (
		<span className="agent-head">
			<span className="page-actions">
				<Dot tone={agentTone(page.status)} label="Status" />
				{page.canRunNow && (
					<Button
						variant="soft"
						className={pulseOn(outcome)}
						disabled={running}
						// on the phone the button is its icon only
						aria-label={phone ? (running ? "Running" : "Run now") : undefined}
						onClick={() => send({ case: "runAgent", value: { agentId } })}
					>
						<Icon name="play" />
						{!phone && (
							<Overlap id={running ? "running" : "run"} className="btn-label">
								{running ? "Running" : "Run now"}
							</Overlap>
						)}
					</Button>
				)}
				<Switch
					on={active}
					label="Agent active"
					onToggle={() =>
						send({
							case: "setAgentActive",
							value: { agentId, active: !active },
						})
					}
				/>
			</span>
			<OutcomeNote outcome={outcome} />
		</span>
	);
}

function Pane({ pane, agentId }: { pane: AgentPage["pane"]; agentId: string }) {
	switch (pane.case) {
		case "chat":
			return <Chat chat={pane.value} agentId={agentId} />;
		case "results":
			return (
				<Scroll>
					<Results results={pane.value} />
				</Scroll>
			);
		case "activity":
			return (
				<Scroll>
					<Activity activity={pane.value} />
				</Scroll>
			);
		case "settings":
			return (
				<Scroll>
					<Configure settings={pane.value} agentId={agentId} />
				</Scroll>
			);
		case undefined:
			return unset("AgentPage.pane");
	}
}

function Scroll({ children }: { children: React.ReactNode }) {
	return (
		<div className="scroll agent-scroll">
			<div className="agent-pane">{children}</div>
		</div>
	);
}

// Approvals first, then the built-in agents and the user's own.
function List({ list }: { list: AgentList }) {
	const send = useSend();
	const touch = useLayout() === "phone";
	const approvals = list.approvals ?? unset("AgentList.approvals");
	return (
		<Page
			label="Agents"
			title="Agents"
			actions={
				<IconButton
					icon="plus"
					label="New agent"
					size={touch ? "lg" : undefined}
					touch={touch}
					onClick={() => send({ case: "openAgentBuilder", value: {} })}
				/>
			}
		>
			<div className="agent-list">
				<button
					type="button"
					className="tile press agent-card approvals-row"
					onClick={() => send({ case: "openApprovals", value: {} })}
				>
					<Icon name="approvals" />
					<span className="grow strong">{approvals.label}</span>
					{approvals.waiting && (
						<span className="sm muted">{approvals.waiting}</span>
					)}
					<Icon name="chevron-right" className="subtle" />
				</button>
				<Group title="Built in" rows={list.systemAgents} first={0} />
				<Group
					title="Yours"
					rows={list.userAgents}
					first={list.systemAgents.length}
				/>
			</div>
		</Page>
	);
}

function Group({
	title,
	rows,
	first,
}: {
	title: string;
	rows: AgentRow[];
	first: number;
}) {
	const send = useSend();
	return (
		<section className="pane-section">
			<h2 className="label">{title}</h2>
			<Rows label={title} className="agent-cards">
				{rows.map((row, i) => (
					<Row key={row.id} index={first + i}>
						<button
							type="button"
							className="tile press agent-card"
							onClick={() =>
								send({ case: "selectAgent", value: { agentId: row.id } })
							}
						>
							<Dot tone={agentTone(row.status)} />
							<span className="grow two-line">
								<span className="trunc strong">{row.name}</span>
								<span className="trunc xs subtle">{row.next}</span>
								<span className="trunc xs subtle">{row.lastRun}</span>
							</span>
							<Icon name="chevron-right" className="subtle" />
						</button>
					</Row>
				))}
			</Rows>
		</section>
	);
}
