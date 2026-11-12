import { unset } from "../../engine";
import {
	type AgentActivity,
	type AgentRun,
	type RunOutcome,
	type RunStep,
	StepPermission,
} from "../../gen/ui/v1/agents_pb";
import { cx } from "../../ui/cx";
import { Dot, type Tone } from "../../ui/Dot";
import { useDisclosure } from "../../ui/disclosure";
import { Icon } from "../../ui/Icon";
import { References } from "../../ui/Reference";
import { Stats } from "../../ui/Stats";
import { stagger } from "../../ui/stagger";

// Stats, then the runs as a log: one dense line per run with its outcome, start, what it did and
// what started it, model, steps, tokens and duration; a run expands to every step with how its
// permission applied and what it touched. It claims no table semantics: the column heads are for
// the eye only (aria-hidden), each run is one button whose values carry their own units.
export function Activity({ activity }: { activity: AgentActivity }) {
	return (
		<div className="pane-sections">
			<Stats stats={activity.stats} />
			<section className="pane-section">
				<h2 className="label">Runs</h2>
				<div className="run-log">
					<div className="run-cols xs subtle" aria-hidden="true">
						<span className="run-lead">Start</span>
						<span>Run</span>
						<span className="run-meta">Model</span>
						<span className="run-meta">Steps</span>
						<span className="run-meta">Tokens</span>
						<span className="run-num">Took</span>
					</div>
					{activity.runs.map((run) => (
						<Run key={run.id} run={run} />
					))}
				</div>
			</section>
		</div>
	);
}

function outcome({ state }: RunOutcome): {
	tone: Tone;
	label: string;
	note: string;
} {
	switch (state.case) {
		case "done":
			return { tone: "done", label: "Done", note: "" };
		case "waiting":
			return { tone: "wait", label: "Waiting for you", note: "" };
		case "running":
			return { tone: "run", label: "Running", note: "" };
		case "failed":
			return { tone: "err", label: "Failed", note: state.value.reason };
		case "stopped":
			return { tone: "idle", label: "Stopped", note: state.value.text };
		case "paused":
			return { tone: "wait", label: "Paused", note: "" };
		case undefined:
			return unset("RunOutcome.state");
	}
}

function Run({ run }: { run: AgentRun }) {
	const { isOpen: open, toggle } = useDisclosure({
		case: "runId",
		value: run.id,
	}).of(run.detail, "AgentRun.detail");
	const result = outcome(run.outcome ?? unset("AgentRun.outcome"));
	return (
		<div className={cx("run", open && "open")}>
			<button
				type="button"
				className="press run-cols run-row"
				aria-expanded={open}
				onClick={toggle}
			>
				<span className="run-lead">
					<Icon
						name="chevron-right"
						size="sm"
						className={cx("subtle chev", open && "open")}
					/>
					<Dot tone={result.tone} label={result.label} />
					<span className="sm tnum">{run.start}</span>
				</span>
				<span className="run-what">
					<span className="trunc sm">{run.summary}</span>
					<span className="trunc xs subtle">
						{(run.cause ?? unset("AgentRun.cause")).text}
					</span>
				</span>
				<span className="run-meta xs muted trunc">{run.model}</span>
				<span className="run-meta xs muted">{run.stepCount}</span>
				<span className="run-meta xs muted tnum">{run.tokens}</span>
				<span className="run-num xs muted tnum">{run.duration}</span>
			</button>
			{open && (
				<div className="reveal run-detail">
					<div className="run-facts xs subtle settle">
						<span>
							{result.label}
							{result.note && ` · ${result.note}`}
						</span>
						<span className="run-facts-wide">
							{run.model} · {run.stepCount} · {run.tokens} · {run.duration}
						</span>
						<References
							references={(run.cause ?? unset("AgentRun.cause")).references}
							className="xs"
						/>
					</div>
					<ol className="run-steps">
						{run.steps.map((step, i) => (
							<Step key={step.id} step={step} index={i} />
						))}
					</ol>
				</div>
			)}
		</div>
	);
}

function permissionTone(permission: StepPermission): Tone | undefined {
	switch (permission) {
		case StepPermission.READ:
			return undefined;
		case StepPermission.AUTO:
			return "done";
		case StepPermission.ASKED:
			return "wait";
		case StepPermission.OFF:
			return "idle";
		case StepPermission.UNSPECIFIED:
			return unset("StepPermission");
	}
}

function Step({ step, index }: { step: RunStep; index: number }) {
	const tone = permissionTone(step.permission);
	return (
		<li className="settle run-step" style={stagger(index)}>
			<span className="xs subtle tnum">{index + 1}</span>
			<span className="run-step-what">
				<span className="sm">
					<span className="strong">{step.verb}</span>{" "}
					<span className="muted">{step.what}</span>
				</span>
				<References references={step.references} className="xs" />
			</span>
			<span className="xs subtle status-line run-permission">
				{tone && <Dot tone={tone} />}
				{step.permissionLabel}
			</span>
		</li>
	);
}
