import { unset } from "../../engine";
import type {
	AgentDetail,
	AgentRun,
	RunOutcome,
	SystemAgent,
} from "../../gen/ui/v1/agents_pb";
import { useDisclosure, useDraft, useLayout } from "../../presentation";
import { Button } from "../../ui/Button";
import { cx } from "../../ui/cx";
import { Dot, type Tone } from "../../ui/Dot";
import { Field } from "../../ui/Field";
import { Icon } from "../../ui/Icon";
import { PermissionLine } from "../../ui/Permission";
import { Switch } from "../../ui/Switch";
import { stagger } from "../../ui/stagger";
import { isActive } from "./status";

// Run now, Edit, Delete and the active switch have no intent yet and are drawn without one.
export function AgentActions({ kind }: { kind: AgentDetail["kind"] }) {
	return (
		<span className="agent-actions">
			<Button variant="soft">
				<Icon name="play" />
				Run now
			</Button>
			<UserActions kind={kind} />
		</span>
	);
}

function UserActions({ kind }: { kind: AgentDetail["kind"] }) {
	switch (kind.case) {
		case "system":
			return null;
		case "user":
			return (
				<>
					<Button variant="ghost" className="agent-edit">
						Edit
					</Button>
					<Button variant="ghost" className="agent-delete">
						Delete
					</Button>
				</>
			);
		case undefined:
			return unset("AgentDetail.kind");
	}
}

// The user's additions to a built-in agent; kept as typed, as no intent saves them yet.
export function YourInstructions({
	system,
	placeholder,
}: {
	system: SystemAgent;
	placeholder: string;
}) {
	const [text, setText] = useDraft(system.yourInstructions);
	return (
		<Field
			label={system.yourInstructionsLabel}
			hideLabel={useLayout() === "phone"}
			rows={2}
			placeholder={placeholder}
			value={text}
			onChange={setText}
			className="agent-extra"
		/>
	);
}

function runTone({ state }: RunOutcome): Tone {
	switch (state.case) {
		case "done":
			return "done";
		case "waiting":
			return "wait";
		case undefined:
			return unset("RunOutcome.state");
	}
}

function Run({ run }: { run: AgentRun }) {
	const { disclosure, toggle } = useDisclosure();
	const open = disclosure === "expanded";
	return (
		<li className="run">
			<button
				type="button"
				className="menu-row press run-head"
				aria-expanded={open}
				onClick={toggle}
			>
				<Icon
					name="chevron-right"
					size="sm"
					className={cx("subtle chev", open && "open")}
				/>
				<Dot
					tone={run.outcome ? runTone(run.outcome) : unset("AgentRun.outcome")}
				/>
				<span className="sm subtle run-time">{run.time}</span>
				<span className="trunc grow">{run.summary}</span>
				<span className="xs subtle">{run.duration}</span>
			</button>
			{open && (
				<div className="reveal run-steps">
					{run.steps.map((step, i) => (
						<span
							// biome-ignore lint/suspicious/noArrayIndexKey: steps carry no id and never reorder
							key={i}
							className="settle sm run-step"
							style={stagger(i)}
						>
							<span className="subtle run-verb">{step.verb}</span>
							<span className="muted">{step.what}</span>
						</span>
					))}
				</div>
			)}
		</li>
	);
}

function Schedule({ system }: { system: SystemAgent }) {
	const { schedule } = system;
	switch (schedule.case) {
		case "onEvent":
		case "onMessage":
			return <span className="sm">{schedule.value.when}</span>;
		case "atTimes":
			return (
				<>
					<span className="sm when">{schedule.value.when}</span>
					{schedule.value.times.map((time) => (
						<button key={time} type="button" className="chip press time-chip">
							{time}
						</button>
					))}
					<button type="button" className="chip press">
						<Icon name="plus" size="sm" />
						Add a time
					</button>
				</>
			);
		case undefined:
			return unset("SystemAgent.schedule");
	}
}

function SystemBody({ system }: { system: SystemAgent }) {
	return (
		<>
			<div className="agent-section">
				<span className="label with-icon">
					<Icon name="lock" size="sm" />
					Built-in instructions, kept up to date by EveryMail
				</span>
				<div className="well muted agent-text">
					{system.builtInInstructions}
				</div>
			</div>
			<YourInstructions
				system={system}
				placeholder="Added after the built-in ones"
			/>
			<div className="agent-section">
				<span className="label">When it runs</span>
				<span className="agent-when">
					<Schedule system={system} />
				</span>
			</div>
			<div className="agent-section">
				<span className="label">What it may do</span>
				{system.permissions.map((permission) => (
					<PermissionLine key={permission.action} permission={permission} />
				))}
				<span className="xs subtle">{system.keyNote}</span>
			</div>
		</>
	);
}

function KindBody({ kind }: { kind: AgentDetail["kind"] }) {
	switch (kind.case) {
		case "system":
			return <SystemBody system={kind.value} />;
		case "user":
			return <div className="well agent-text">{kind.value.instructions}</div>;
		case undefined:
			return unset("AgentDetail.kind");
	}
}

// The selected agent's name, what it is and its controls.
export function DetailHead({ detail }: { detail: AgentDetail }) {
	return (
		<div className="agent-head">
			<span className="agent-title">
				<h1 className="h1">{detail.name}</h1>
				<span className="sm muted">{detail.subtitle}</span>
			</span>
			<AgentActions kind={detail.kind} />
			<Switch on={isActive(detail.status)} label="Agent active" />
		</div>
	);
}

// What the agent did, as drawn: what it is, figures and recent runs.
export function Activity({ detail }: { detail: AgentDetail }) {
	return (
		<>
			<KindBody kind={detail.kind} />
			<div className="agent-stats">
				{detail.stats.map((stat) => (
					<div key={stat.label} className="tile stat">
						<span className="stat-value">{stat.value}</span>
						<span className="xs subtle">{stat.label}</span>
					</div>
				))}
			</div>
			<div className="agent-section">
				<span className="label">Recent runs</span>
				<ul className="runs" aria-label="Recent runs">
					{detail.runs.map((run) => (
						<Run key={run.id} run={run} />
					))}
				</ul>
			</div>
		</>
	);
}
