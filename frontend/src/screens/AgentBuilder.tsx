import "./agent-builder.css";
import { type ReactNode, useId } from "react";
import { unset, useSend } from "../engine";
import {
	type AgentBuilderView,
	type BuilderSchedule,
	BuilderStep,
	type BuilderStepTab,
	type BuilderTrigger,
	type DryRunResult,
} from "../gen/ui/v1/agent_builder_pb";
import { useDraft, useLayout } from "../presentation";
import { Button, IconButton } from "../ui/Button";
import { Check } from "../ui/Check";
import { cx } from "../ui/cx";
import { Dot, type Tone } from "../ui/Dot";
import { Field } from "../ui/Field";
import { Icon } from "../ui/Icon";
import { SwitchLine } from "../ui/Lines";
import { Logo } from "../ui/Logo";
import { Overlap } from "../ui/Overlap";
import { Popover } from "../ui/Overlay";
import { PermissionLine } from "../ui/Permission";
import { Row, Rows } from "../ui/Rows";
import { SelectionMark, useSelectionGroup } from "../ui/Selection";
import { Switch } from "../ui/Switch";
import { stagger } from "../ui/stagger";
import type { ScreenProps } from "./Screen";

// A new agent being written. Back, Cancel and the phone's Close are the back intent; Continue and
// Back on the phone walk the engine's steps. Every field's commit, the ideas, adding and removing
// triggers, the schedule, the account and recipient switches, Run a test and Create agent have no
// intent yet and are drawn without one; the name and instructions stay in their fields.
export function AgentBuilder({ view }: ScreenProps<AgentBuilderView>) {
	const layout = useLayout();
	switch (layout) {
		case "phone":
			return <PhoneBuilder view={view} />;
		case "list":
			return <DesktopBuilder view={view} />;
		case "split":
			return (
				<>
					<DesktopBuilder view={view} />
					<TestAside view={view} />
				</>
			);
	}
}

const back = { case: "back", value: {} } as const;

function scheduleOf(view: AgentBuilderView) {
	return view.schedule ?? unset("AgentBuilderView.schedule");
}

function NameField({ view }: { view: AgentBuilderView }) {
	const [name, setName] = useDraft(view.name);
	return <Field label="Name" value={name} onChange={setName} />;
}

function InstructionsField({
	view,
	label,
	hint,
}: {
	view: AgentBuilderView;
	label: string;
	hint?: string;
}) {
	const id = useId();
	const [text, setText] = useDraft(view.instructions);
	return (
		<div className="builder-field">
			<label htmlFor={id} className="label instructions-label">
				{label}
			</label>
			<textarea
				id={id}
				className="builder-text"
				rows={6}
				value={text}
				onChange={(event) => setText(event.target.value)}
			/>
			{hint && <span className="xs subtle">{hint}</span>}
		</div>
	);
}

function Accounts({ view }: { view: AgentBuilderView }) {
	return (
		<div className="builder-field tight">
			<span className="label">Accounts it can see</span>
			{view.accounts.map((account) => (
				<div key={account.id} className="builder-account">
					<Logo
						brand={{ kind: "mail", provider: account.provider }}
						size={20}
					/>
					<span className="grow two-line">
						<span>{account.name}</span>
						<span className="xs subtle">{account.providerName}</span>
					</span>
					<Switch on={account.canSee} label={account.name} />
				</div>
			))}
		</div>
	);
}

// On the desktop form, under its label and with the new-recipient switch (`children`); on the
// phone, its own step, whose title names it.
function Permissions({
	view,
	label,
	children,
}: {
	view: AgentBuilderView;
	label?: string;
	children?: ReactNode;
}) {
	return (
		<div className="builder-field tight">
			{label && <span className="label">{label}</span>}
			<span className="sm subtle permissions-intro">
				Auto acts on its own. Ask sends it to your approval queue first. Off
				hides the action from the agent
			</span>
			{view.permissions.map((permission) => (
				<PermissionLine key={permission.action} permission={permission} />
			))}
			{children}
		</div>
	);
}

// desktop

function DesktopBuilder({ view }: { view: AgentBuilderView }) {
	const send = useSend();
	return (
		<section className="panel builder-main" aria-label="New agent">
			<div className="builder-bar">
				<IconButton
					icon="chevron-left"
					label="Back to agents"
					onClick={() => send(back)}
				/>
				<h1 className="h2 grow">New agent</h1>
				<Button variant="ghost" className="cancel" onClick={() => send(back)}>
					Cancel
				</Button>
				<Button variant="primary" className="create">
					Create agent
				</Button>
			</div>
			<div className="scroll builder-scroll">
				<div className="builder-form">
					<NameField view={view} />
					<InstructionsField
						view={view}
						label="What should it do"
						hint="Plain words work. Mention people, folders, sheets and calendars by name"
					/>
					<Triggers view={view} />
					<Accounts view={view} />
					<Permissions view={view} label="What it may do">
						<SwitchLine
							label="Always ask before writing to someone new"
							hint="Overrides Auto for first-time recipients"
							on={view.askBeforeNewRecipient}
						/>
					</Permissions>
				</div>
			</div>
		</section>
	);
}

function Filters({ trigger }: { trigger: BuilderTrigger }) {
	return (
		<span className="trigger-filters">
			{trigger.filters.map((filter) => (
				<button
					key={filter.key}
					type="button"
					className="chip press filter-chip"
				>
					<span className="subtle">{filter.key}</span>
					{filter.value}
				</button>
			))}
		</span>
	);
}

function Triggers({ view }: { view: AgentBuilderView }) {
	const chosen = view.triggers.filter((trigger) => trigger.chosen);
	const free = view.triggers.filter((trigger) => !trigger.chosen);
	const schedule = scheduleOf(view);
	return (
		<div className="builder-field">
			<span className="label">When it runs</span>
			<Rows label="Triggers" className="trigger-rows">
				{chosen.map((trigger, i) => (
					<Row key={trigger.id} index={i} className="tile trigger">
						<TriggerHead label={trigger.label} kind="When something happens" />
						<Filters trigger={trigger} />
					</Row>
				))}
				{schedule.state.case === "scheduled" && (
					<Row key="schedule" index={chosen.length} className="tile trigger">
						<TriggerHead label="Schedule" kind="On a schedule" />
						<ScheduleChoices
							schedule={schedule}
							label={schedule.state.value.label}
						/>
					</Row>
				)}
			</Rows>
			<span className="add-trigger">
				<Popover
					label="Add a trigger"
					className="add-pop"
					trigger={(props) => (
						<Button variant="soft" className="add-button" {...props}>
							<Icon name="plus" size="sm" />
							Add a trigger
						</Button>
					)}
				>
					{() => (
						<>
							<span className="label pop-label">When something happens</span>
							{free.map((trigger, i) => (
								<button
									key={trigger.id}
									type="button"
									role="menuitem"
									className="menu-row row-in stagger"
									style={stagger(i)}
								>
									{trigger.label}
								</button>
							))}
							<AddSchedule schedule={schedule} />
						</>
					)}
				</Popover>
			</span>
			<span className="xs subtle">
				Filters are checked before the model runs, so mail that doesn’t match
				costs nothing
			</span>
		</div>
	);
}

function AddSchedule({ schedule }: { schedule: BuilderSchedule }) {
	switch (schedule.state.case) {
		case "none":
			return (
				<>
					<span className="label pop-label second">On a schedule</span>
					<button type="button" role="menuitem" className="menu-row row-in">
						Schedule
					</button>
				</>
			);
		case "scheduled":
			return null;
		case undefined:
			return unset("BuilderSchedule.state");
	}
}

function TriggerHead({ label, kind }: { label: string; kind: string }) {
	return (
		<span className="trigger-head">
			<span className="grow trigger-name">
				<span className="strong">{label}</span>
				<span className="xs subtle">{kind}</span>
			</span>
			<IconButton icon="close" size="sm" label="Remove trigger" />
		</span>
	);
}

function ScheduleChoices({
	schedule,
	label,
}: {
	schedule: BuilderSchedule;
	label: string;
}) {
	return (
		<span className="schedule-line">
			<Popover
				label="Schedule"
				className="schedule-pop"
				trigger={(props) => (
					<button type="button" className="chip press schedule-chip" {...props}>
						{label}
						<Icon name="chevron-down" size="sm" className="subtle" />
					</button>
				)}
			>
				{() =>
					schedule.choices.map((choice, i) => (
						<button
							key={choice.label}
							type="button"
							role="menuitemradio"
							aria-checked={choice.selected}
							className="menu-row row-in stagger"
							style={stagger(i)}
						>
							<span className="grow">{choice.label}</span>
							<Check on={choice.selected} />
						</button>
					))
				}
			</Popover>
			<span className="xs subtle">
				Every 15 minutes is the shortest. Runs in the cloud, devices can be off
			</span>
		</span>
	);
}

// the dry run

function decisionOf({ decision }: DryRunResult): { tone: Tone; name: string } {
	switch (decision.case) {
		case "ask":
			return { tone: "wait", name: "Ask" };
		case "auto":
			return { tone: "done", name: "Auto" };
		case "skip":
			return { tone: "idle", name: "Skip" };
		case undefined:
			return unset("DryRunResult.decision");
	}
}

function DryRun({ view }: { view: AgentBuilderView }) {
	const { dryRun } = view;
	switch (dryRun.case) {
		case "idle":
			return (
				<div className="dry-note">
					<span className="sm subtle">
						Results show here, one row per message it would touch
					</span>
				</div>
			);
		case "reading":
			return (
				<div className="dry-reading">
					<span className="shimmer sm">{dryRun.value.progress}</span>
				</div>
			);
		case "results":
			return (
				<div className="dry-results">
					{dryRun.value.results.map((result, i) => {
						const { tone, name } = decisionOf(result);
						return (
							<div
								// biome-ignore lint/suspicious/noArrayIndexKey: results carry no id and never reorder
								key={i}
								className="settle dry-result"
								style={stagger(i)}
							>
								<span className="dry-line">
									<Dot tone={tone} />
									<span className="trunc grow strong">{result.from}</span>
									<span className={cx("xs", `dry-${tone}`)}>{name}</span>
								</span>
								<span className="sm muted dry-action">{result.action}</span>
							</div>
						);
					})}
				</div>
			);
		case undefined:
			return unset("AgentBuilderView.dry_run");
	}
}

function TestButton({ view }: { view: AgentBuilderView }) {
	const again = view.dryRun.case === "results";
	const label = () => {
		switch (view.dryRun.case) {
			case "idle":
				return "Run a test";
			case "reading":
				return "Testing";
			case "results":
				return "Run again";
			case undefined:
				return unset("AgentBuilderView.dry_run");
		}
	};
	return (
		<Button
			variant="soft"
			size="lg"
			className={cx("test-button", again && "pulse")}
		>
			<span className="swap">
				<Icon name="play" className={cx(again && "hide")} />
				<Icon name="retry" className={cx(!again && "hide")} />
			</span>
			{label()}
		</Button>
	);
}

function TestAside({ view }: { view: AgentBuilderView }) {
	const { testModel } = view;
	return (
		<aside className="panel builder-test" aria-label="Test run">
			<div className="test-head">
				<div className="test-title">
					<h2 className="h2 grow">Try it first</h2>
					{testModel && (
						<span className="chip test-model">
							<Logo
								brand={{ kind: "model", provider: testModel.provider }}
								size={16}
							/>
							{testModel.name}
						</span>
					)}
				</div>
				<span className="sm muted">
					A dry run on your last 30 messages. Nothing is sent, moved or changed
				</span>
				<TestButton view={view} />
			</div>
			<div className="scroll test-scroll">
				<Overlap id={view.dryRun.case ?? unset("AgentBuilderView.dry_run")}>
					<DryRun view={view} />
				</Overlap>
			</div>
		</aside>
	);
}

// phone

function stepTabs(view: AgentBuilderView) {
	const at = view.steps.findIndex((tab) => tab.current);
	const current = view.steps[at];
	if (!current) return unset("AgentBuilderView.steps");
	return {
		at,
		current,
		previous: view.steps[at - 1],
		next: view.steps[at + 1],
	};
}

function PhoneBuilder({ view }: { view: AgentBuilderView }) {
	const send = useSend();
	const { at, current, previous, next } = stepTabs(view);
	const go = (tab: BuilderStepTab) => () =>
		send({ case: "selectBuilderStep", value: { step: tab.step } });
	return (
		<div className="builder-phone">
			<header className="builder-phone-head">
				<div className="builder-phone-bar">
					{previous ? (
						<IconButton
							icon="chevron-left"
							label="Back"
							size="lg"
							touch
							onClick={go(previous)}
						/>
					) : (
						<IconButton
							icon="close"
							label="Close"
							size="lg"
							touch
							onClick={() => send(back)}
						/>
					)}
					<h2 className="h2 grow builder-phone-title">New agent</h2>
				</div>
				<div className="bar builder-progress">
					<i style={{ width: `${((at + 1) / view.steps.length) * 100}%` }} />
				</div>
			</header>
			<Overlap id={String(current.step)} className="builder-phone-stack">
				<div className="scroll noscroll builder-phone-step">
					<Step view={view} tab={current} />
				</div>
			</Overlap>
			<div className="builder-phone-foot">
				{next ? (
					<Button variant="primary" size="lg" onClick={go(next)}>
						Continue
					</Button>
				) : (
					<Button variant="primary" size="lg">
						Create agent
					</Button>
				)}
			</div>
		</div>
	);
}

function Step({ view, tab }: { view: AgentBuilderView; tab: BuilderStepTab }) {
	switch (tab.step) {
		case BuilderStep.INSTRUCTIONS:
			return (
				<>
					<h1 className="h1">{tab.label}</h1>
					<InstructionsField view={view} label="Instructions" />
					<span className="label idea-label">Start from an idea</span>
					{view.ideas.map((idea, i) => (
						<button
							key={idea}
							type="button"
							className="menu-row touch press row-in stagger idea"
							style={stagger(i)}
						>
							<span className="grow">{idea}</span>
							<Icon name="plus" size="sm" className="subtle" />
						</button>
					))}
				</>
			);
		case BuilderStep.TRIGGERS:
			return <PhoneTriggers view={view} label={tab.label} />;
		case BuilderStep.PERMISSIONS:
			return (
				<>
					<h1 className="h1">{tab.label}</h1>
					<Permissions view={view} />
				</>
			);
		case BuilderStep.SUMMARY:
			return (
				<>
					<h1 className="h1">{view.name}</h1>
					<div className="tile summary-lines">
						{view.summary.map((line, i) => (
							<div
								key={line.label}
								className="settle summary-line"
								style={stagger(i)}
							>
								<span className="subtle summary-label">{line.label}</span>
								<span className="grow">{line.value}</span>
							</div>
						))}
					</div>
					<span className="sm subtle">
						You can change any of this later from Agents
					</span>
				</>
			);
		case BuilderStep.UNSPECIFIED:
			return unset("BuilderStep");
	}
}

function PhoneTriggers({
	view,
	label,
}: {
	view: AgentBuilderView;
	label: string;
}) {
	const group = useSelectionGroup();
	return (
		<>
			<h1 className="h1">{label}</h1>
			<span className="sm subtle pick-any">Pick any number</span>
			<span className="label">When something happens</span>
			<div className="tile trigger-list">
				{view.triggers.map((trigger) => (
					<div key={trigger.id} className="trigger-item">
						{/* biome-ignore lint/a11y/useSemanticElements: as drawn, a trigger is a button row */}
						<button
							type="button"
							role="checkbox"
							aria-checked={trigger.chosen}
							className="menu-row touch press"
						>
							<span className="grow">{trigger.label}</span>
							<Check on={trigger.chosen} />
						</button>
						{trigger.chosen && (
							<span className="reveal xs subtle filter-summary">
								{trigger.filterSummary}
							</span>
						)}
					</div>
				))}
			</div>
			<span className="label">On a schedule</span>
			<div
				className="tile schedule-list"
				role="radiogroup"
				aria-label="Schedule"
			>
				{scheduleOf(view).choices.map((choice) => (
					// biome-ignore lint/a11y/useSemanticElements: as drawn, a choice is a button row
					<button
						key={choice.label}
						type="button"
						role="radio"
						aria-checked={choice.selected}
						className="menu-row touch press selectable"
					>
						{choice.selected && <SelectionMark group={group} />}
						<span className="grow">{choice.label}</span>
						<Check on={choice.selected} />
					</button>
				))}
			</div>
			<span className="sm subtle">
				Every 15 minutes is the shortest. Runs in the cloud, your phone can be
				off
			</span>
		</>
	);
}
