import "./agent-builder.css";
import type { ReactNode } from "react";
import { type IntentKind, match, unset, useSend } from "../engine";
import {
	type AgentBuilderView,
	type BuilderSchedule,
	BuilderStep,
	type BuilderStepTab,
	type BuilderTrigger,
	type DryRunResult,
} from "../gen/ui/v1/agent_builder_pb";
import { Form } from "../gen/ui/v1/common_pb";
import { useDebounce, useDraft, useLayout } from "../presentation";
import { Button, IconButton } from "../ui/Button";
import { Check } from "../ui/Check";
import { Confirm } from "../ui/Confirm";
import { cx } from "../ui/cx";
import { Dot, type Tone } from "../ui/Dot";
import { Field } from "../ui/Field";
import { Icon } from "../ui/Icon";
import { SwitchLine } from "../ui/Lines";
import { Logo } from "../ui/Logo";
import { OutcomeNote, pulseOn, useOutcome } from "../ui/Outcome";
import { Overlap } from "../ui/Overlap";
import { Popover } from "../ui/Overlay";
import { PermissionLine } from "../ui/Permission";
import { PhoneHead } from "../ui/PhoneHead";
import { Row, Rows } from "../ui/Rows";
import { SelectionMark, useSelectionGroup } from "../ui/Selection";
import { Switch } from "../ui/Switch";
import { stagger } from "../ui/stagger";
import type { ScreenProps } from "./Screen";

// A new agent being written. Back, Cancel and the phone's Close are the back intent; Continue and
// Back on the phone walk the engine's steps. Every control sends its intent; the name and the
// instructions go on a debounce. Create is enabled once the engine says the agent is complete.
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

// The name and the instructions as typed. Both go in one update_builder on one debounce, so moving
// quickly between the fields never sends one with the other's old text. The engine replaces both
// (an idea picked, Start over) with a new revision, which the caller keys this by. The phone shows
// the instructions only (`name` absent); the name it sends is the engine's.
function BuilderTexts({
	view,
	name,
	instructions,
}: {
	view: AgentBuilderView;
	name?: { label: string };
	instructions: {
		label: string;
		hint?: string;
		hideLabel?: boolean;
	};
}) {
	const send = useSend();
	const [text, setText] = useDraft({
		name: view.name,
		instructions: view.instructions,
	});
	// Leaving a field sends what is pending, before whatever took the focus acts (Start over, an
	// idea), so the engine never gets old text after it.
	const { schedule, flush } = useDebounce();
	const change = (next: { name: string; instructions: string }) => {
		setText(next);
		schedule(() => send({ case: "updateBuilder", value: next }));
	};
	return (
		<>
			{name && (
				<Field
					label={name.label}
					value={text.name}
					onBlur={flush}
					onChange={(value) => change({ ...text, name: value })}
				/>
			)}
			<Field
				{...instructions}
				rows={6}
				className="builder-text"
				value={text.instructions}
				onBlur={flush}
				onChange={(value) => change({ ...text, instructions: value })}
			/>
		</>
	);
}

function Accounts({ view }: { view: AgentBuilderView }) {
	const send = useSend();
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
					<Switch
						on={account.canSee}
						label={account.name}
						onToggle={() =>
							send({
								case: "setBuilderAccount",
								value: { accountId: account.id, canSee: !account.canSee },
							})
						}
					/>
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
	const send = useSend();
	return (
		<div className="builder-field tight">
			{label && <span className="label">{label}</span>}
			<span className="sm subtle permissions-intro">
				Auto acts on its own. Ask sends it to your approval queue first. Off
				hides the action from the agent
			</span>
			{view.permissions.map((permission) => (
				<PermissionLine
					key={permission.id}
					permission={permission}
					onLevel={(level) =>
						send({
							case: "setBuilderPermission",
							value: { permissionId: permission.id, level },
						})
					}
				/>
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
				<StartOver />
				<Button variant="ghost" className="cancel" onClick={() => send(back)}>
					Cancel
				</Button>
				<CreateButton view={view} />
			</div>
			<Creation view={view} />
			<div className="scroll builder-scroll">
				<div className="builder-form">
					<BuilderTexts
						key={view.revision}
						view={view}
						name={{ label: "Name" }}
						instructions={{
							label: "What should it do",
							hint: "Plain words work. Mention people, folders, sheets and calendars by name",
						}}
					/>
					<Triggers view={view} />
					<Accounts view={view} />
					<Permissions view={view} label="What it may do">
						<SwitchLine
							label="Always ask before writing to someone new"
							hint="Overrides Auto for first-time recipients"
							on={view.askBeforeNewRecipient}
							onToggle={() =>
								send({
									case: "setAskBeforeNewRecipient",
									value: { ask: !view.askBeforeNewRecipient },
								})
							}
						/>
					</Permissions>
				</div>
			</div>
		</section>
	);
}

// Each filter is a menu of its choices, the picked one marked; a required filter nobody picked
// shows its placeholder, and a chosen trigger missing one says which.
function Filters({ trigger }: { trigger: BuilderTrigger }) {
	const send = useSend();
	return (
		<>
			<span className="trigger-filters">
				{trigger.filters.map((filter) => (
					<Popover
						key={filter.id}
						label={filter.label}
						className="filter-pop"
						trigger={(props) => (
							<button
								type="button"
								{...props}
								className="chip press filter-chip"
							>
								<span className="subtle">{filter.label}</span>
								{match(filter.pick, "TriggerFilter.pick", {
									picked: (picked) => picked.label,
									notPicked: (none) => (
										<span className="perm-warn">{none.placeholder}</span>
									),
								})}
								<Icon name="chevron-down" size="sm" className="subtle" />
							</button>
						)}
					>
						{(dismiss) =>
							filter.choices.map((choice, i) => (
								<button
									key={choice.id}
									type="button"
									role="menuitemradio"
									aria-checked={choice.selected}
									className="menu-row press row-in"
									style={stagger(i)}
									onClick={() => {
										dismiss();
										send({
											case: "setTriggerFilter",
											value: {
												triggerId: trigger.id,
												filterId: filter.id,
												choiceId: choice.id,
											},
										});
									}}
								>
									<span className="grow">{choice.label}</span>
									<Check on={choice.selected} />
								</button>
							))
						}
					</Popover>
				))}
			</span>
			<Readiness trigger={trigger} />
		</>
	);
}

function Readiness({ trigger }: { trigger: BuilderTrigger }) {
	const { readiness } = trigger;
	switch (readiness.case) {
		case "ready":
			return null;
		case "incomplete":
			return (
				<span className="xs perm-warn settle">{readiness.value.text}</span>
			);
		case undefined:
			return unset("BuilderTrigger.readiness");
	}
}

const chooseTrigger = (triggerId: string, chosen: boolean) =>
	({ case: "setBuilderTrigger", value: { triggerId, chosen } }) as const;

function Triggers({ view }: { view: AgentBuilderView }) {
	const send = useSend();
	const chosen = view.triggers.filter((trigger) => trigger.chosen);
	const free = view.triggers.filter((trigger) => !trigger.chosen);
	const schedule = scheduleOf(view);
	return (
		<div className="builder-field">
			<span className="label">When it runs</span>
			<Rows label="Triggers" className="trigger-rows">
				{chosen.map((trigger, i) => (
					<Row key={trigger.id} index={i} className="tile trigger">
						<TriggerHead
							label={trigger.label}
							kind="When something happens"
							intent={chooseTrigger(trigger.id, false)}
						/>
						<Filters trigger={trigger} />
					</Row>
				))}
				{match(schedule.state, "BuilderSchedule.state", {
					none: () => null,
					scheduled: (scheduled) => (
						<Row key="schedule" index={chosen.length} className="tile trigger">
							<TriggerHead
								label="Schedule"
								kind="On a schedule"
								intent={pickSchedule(noSchedule)}
							/>
							<ScheduleChoices schedule={schedule} label={scheduled.label} />
						</Row>
					),
				})}
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
					{(dismiss) => (
						<>
							<span className="label pop-label">When something happens</span>
							{free.map((trigger, i) => (
								<button
									key={trigger.id}
									type="button"
									role="menuitem"
									className="menu-row row-in"
									style={stagger(i)}
									onClick={() => {
										dismiss();
										send(chooseTrigger(trigger.id, true));
									}}
								>
									{trigger.label}
								</button>
							))}
							<AddSchedule schedule={schedule} dismiss={dismiss} />
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

// A schedule choice by its id; "none" is the contract's id of No schedule (BuilderSchedule.choices),
// and "custom" the one whose interval the user types.
const noSchedule = "none";
const customSchedule = "custom";
const pickSchedule = (choiceId: string) =>
	({ case: "setBuilderSchedule", value: { choiceId } }) as const;

// Unscheduled, the menu offers every schedule but No schedule.
function AddSchedule({
	schedule,
	dismiss,
}: {
	schedule: BuilderSchedule;
	dismiss: () => void;
}) {
	const send = useSend();
	switch (schedule.state.case) {
		case "none":
			return (
				<>
					<span className="label pop-label second">On a schedule</span>
					{schedule.choices
						.filter((choice) => choice.id !== noSchedule)
						.map((choice) => (
							<button
								key={choice.id}
								type="button"
								role="menuitem"
								className="menu-row row-in"
								onClick={() => {
									dismiss();
									send(pickSchedule(choice.id));
								}}
							>
								{choice.label}
							</button>
						))}
				</>
			);
		case "scheduled":
			return null;
		case undefined:
			return unset("BuilderSchedule.state");
	}
}

function TriggerHead({
	label,
	kind,
	intent,
}: {
	label: string;
	kind: string;
	intent: IntentKind;
}) {
	const send = useSend();
	return (
		<span className="trigger-head">
			<span className="grow trigger-name">
				<span className="strong">{label}</span>
				<span className="xs subtle">{kind}</span>
			</span>
			<IconButton
				icon="close"
				size="sm"
				label={`Remove ${label}`}
				onClick={() => send(intent)}
			/>
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
	const send = useSend();
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
				{(dismiss) =>
					schedule.choices.map((choice, i) => (
						<button
							key={choice.id}
							type="button"
							role="menuitemradio"
							aria-checked={choice.selected}
							className="menu-row row-in"
							style={stagger(i)}
							onClick={() => {
								dismiss();
								send(pickSchedule(choice.id));
							}}
						>
							<span className="grow">{choice.label}</span>
							<Check on={choice.selected} />
						</button>
					))
				}
			</Popover>
			<CustomInterval schedule={schedule} />
			<span className="xs subtle">Runs in the cloud, devices can be off</span>
		</span>
	);
}

// The interval field, while the custom schedule is picked. It starts over from the engine's value
// each time that value changes.
function CustomInterval({ schedule }: { schedule: BuilderSchedule }) {
	const custom = schedule.choices.some(
		(choice) => choice.selected && choice.id === customSchedule,
	);
	return (
		custom && <Interval key={schedule.intervalMinutes} schedule={schedule} />
	);
}

// The custom schedule's interval in minutes, sent as typed; the engine says whether it can run that
// often (the hint, and a refusal under the field).
function Interval({ schedule }: { schedule: BuilderSchedule }) {
	const send = useSend();
	const [minutes, setMinutes] = useDraft(schedule.intervalMinutes);
	const outcome = useOutcome({ case: "form", value: Form.BUILDER_INTERVAL });
	const set = () => send({ case: "setBuilderInterval", value: { minutes } });
	return (
		<span className="builder-interval">
			<span className="builder-interval-row">
				<Field
					label="Custom interval, in minutes"
					value={minutes}
					onChange={setMinutes}
					onEnter={set}
				/>
				<Button
					variant="soft"
					size="lg"
					className={pulseOn(outcome)}
					onClick={set}
				>
					Set
				</Button>
			</span>
			<span className="xs subtle">{schedule.intervalHint}</span>
			<OutcomeNote outcome={outcome} />
		</span>
	);
}

// Create, enabled when the engine says the agent is complete; otherwise what is missing.
function CreateButton({ view, size }: { view: AgentBuilderView; size?: "lg" }) {
	const send = useSend();
	return (
		<Button
			variant="primary"
			size={size}
			className="create"
			disabled={match(view.creation, "AgentBuilderView.creation", {
				ready: () => false,
				incomplete: () => true,
			})}
			onClick={() => send({ case: "createAgent", value: {} })}
		>
			Create agent
		</Button>
	);
}

function Creation({ view }: { view: AgentBuilderView }) {
	const outcome = useOutcome({ case: "form", value: Form.AGENT_BUILDER });
	const { creation } = view;
	switch (creation.case) {
		case "ready":
			return <OutcomeNote outcome={outcome} className="builder-creation" />;
		case "incomplete":
			return (
				<span className="xs subtle builder-creation settle">
					{creation.value.text}
				</span>
			);
		case undefined:
			return unset("AgentBuilderView.creation");
	}
}

// Start over, after asking.
function StartOver() {
	const send = useSend();
	return (
		<Confirm
			question="Start this agent over?"
			action="Start over"
			align="end"
			onConfirm={() => send({ case: "discardBuilder", value: {} })}
			trigger={(props) => (
				<Button {...props} variant="ghost">
					Start over
				</Button>
			)}
		/>
	);
}

// the dry run

// The decision's dot, word and the colour of the word.
function decisionOf({ decision }: DryRunResult): {
	tone: Tone;
	name: string;
	colour: "warn" | "ok" | "subtle";
} {
	switch (decision.case) {
		case "ask":
			return { tone: "wait", name: "Ask", colour: "warn" };
		case "auto":
			return { tone: "done", name: "Auto", colour: "ok" };
		case "skip":
			return { tone: "idle", name: "Skip", colour: "subtle" };
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
		case "failed":
			return (
				<div className="dry-note">
					<span className="sm bad" role="alert">
						{dryRun.value.reason}
					</span>
				</div>
			);
		case "results":
			return (
				<div className="dry-results">
					{dryRun.value.results.map((result, i) => {
						const { tone, name, colour } = decisionOf(result);
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
									<span className={cx("xs", colour)}>{name}</span>
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

// Run a test, Run again after results or a failure, and Stop while it reads.
function TestButton({ view }: { view: AgentBuilderView }) {
	const send = useSend();
	const { label, again, intent } = match(
		view.dryRun,
		"AgentBuilderView.dry_run",
		{
			idle: () => ({ label: "Run a test", again: false, intent: start }),
			reading: () => ({ label: "Stop", again: false, intent: stop }),
			results: () => ({ label: "Run again", again: true, intent: start }),
			failed: () => ({ label: "Run again", again: true, intent: start }),
		},
	);
	return (
		<Button
			variant="soft"
			size="lg"
			className="test-button"
			onClick={() => send(intent)}
		>
			<span className="swap">
				<Icon name="play" className={cx(again && "hide")} />
				<Icon name="retry" className={cx(!again && "hide")} />
			</span>
			{label}
		</Button>
	);
}

const start = { case: "startDryRun", value: {} } as const;
const stop = { case: "cancelDryRun", value: {} } as const;

function TestAside({ view }: { view: AgentBuilderView }) {
	const { testModel } = view;
	return (
		<aside className="panel builder-test" aria-label="Test run">
			<div className="test-head">
				<div className="test-title">
					<h2 className="h2 grow">Try it first</h2>
					{testModel && (
						<span className="sm test-model">
							<Logo
								brand={{ kind: "model", provider: testModel.provider }}
								size={16}
							/>
							{testModel.name}
						</span>
					)}
				</div>
				{match(view.dryRun, "AgentBuilderView.dry_run", {
					idle: (idle) => <span className="sm muted">{idle.text}</span>,
					reading: () => null,
					results: () => null,
					failed: () => null,
				})}
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

// The narrow layout's steps: Back goes to the engine's previous step or, on the first, leaves the
// builder; Continue goes to the next step, or the summary shows Create.
function PhoneBuilder({ view }: { view: AgentBuilderView }) {
	const send = useSend();
	const current =
		view.steps.find((tab) => tab.current) ?? unset("AgentBuilderView.steps");
	const go = (step: BuilderStep) => () =>
		send({ case: "selectBuilderStep", value: { step } });
	return (
		<div className="builder-phone">
			<PhoneHead
				bar={
					<>
						{match(view.back, "AgentBuilderView.back", {
							backStep: (step) => (
								<IconButton
									icon="chevron-left"
									label="Back"
									size="lg"
									touch
									onClick={go(step)}
								/>
							),
							first: () => (
								<IconButton
									icon="close"
									label="Close"
									size="lg"
									touch
									onClick={() => send(back)}
								/>
							),
						})}
						<h2 className="h2 grow builder-phone-title">New agent</h2>
					</>
				}
			>
				<div className="bar builder-progress">
					<i style={{ width: `${view.progressPercent}%` }} />
				</div>
			</PhoneHead>
			<Overlap id={String(current.step)} className="builder-phone-stack">
				<div className="scroll noscroll builder-phone-step">
					<Step view={view} tab={current} />
				</div>
			</Overlap>
			<div className="builder-phone-foot">
				{match(view.forward, "AgentBuilderView.forward", {
					nextStep: (step) => (
						<Button variant="primary" size="lg" onClick={go(step)}>
							Continue
						</Button>
					),
					last: () => (
						<>
							<Creation view={view} />
							<CreateButton view={view} size="lg" />
						</>
					),
				})}
			</div>
		</div>
	);
}

function Step({ view, tab }: { view: AgentBuilderView; tab: BuilderStepTab }) {
	const send = useSend();
	switch (tab.step) {
		case BuilderStep.INSTRUCTIONS:
			return (
				<>
					<h1 className="h1">{tab.label}</h1>
					<BuilderTexts
						key={view.revision}
						view={view}
						instructions={{ label: "Instructions", hideLabel: true }}
					/>
					<span className="label idea-label">Start from an idea</span>
					{view.ideas.map((idea, i) => (
						<button
							key={idea}
							type="button"
							className="menu-row touch press row-in idea"
							style={stagger(i)}
							onClick={() => send({ case: "pickIdea", value: { idea } })}
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
	const send = useSend();
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
							onClick={() => send(chooseTrigger(trigger.id, !trigger.chosen))}
						>
							<span className="grow">{trigger.label}</span>
							<Check on={trigger.chosen} />
						</button>
						{trigger.chosen && (
							<span className="reveal filter-summary">
								<span className="xs subtle">{trigger.filterSummary}</span>
								<Filters trigger={trigger} />
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
						key={choice.id}
						type="button"
						role="radio"
						aria-checked={choice.selected}
						className="menu-row touch press selectable"
						onClick={() => send(pickSchedule(choice.id))}
					>
						{choice.selected && <SelectionMark group={group} />}
						<span className="grow">{choice.label}</span>
						<Check on={choice.selected} />
					</button>
				))}
			</div>
			<CustomInterval schedule={scheduleOf(view)} />
			<span className="sm subtle">
				Runs in the cloud, your phone can be off
			</span>
		</>
	);
}
