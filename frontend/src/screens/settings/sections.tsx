import { create } from "@bufbuild/protobuf";
import { type ReactNode, useId } from "react";
import { match, unset, useSend } from "../../engine";
import { Form } from "../../gen/ui/v1/common_pb";
import {
	type AccountsSettings,
	type AgentPermissionsSettings,
	type AppearanceSettings,
	type BlockListSettings,
	type ModelsSettings,
	type ProviderKey,
	type RuleActionChoice,
	type RuleCondition,
	type RuleConditionChoice,
	RuleConditionSchema,
	type RulesSettings,
	type SettingsAccount,
	type TaskModel,
	type YourDataSettings,
} from "../../gen/ui/v1/settings_pb";
import { useDraft } from "../../presentation";
import { Button, IconButton } from "../../ui/Button";
import { Confirm } from "../../ui/Confirm";
import { cx } from "../../ui/cx";
import { accountStatus, Dot } from "../../ui/Dot";
import { useDisclosure } from "../../ui/disclosure";
import { Field } from "../../ui/Field";
import { Icon } from "../../ui/Icon";
import { SettingLine, SwitchLine } from "../../ui/Lines";
import { Logo, LogoTile, ModelChip } from "../../ui/Logo";
import { choiceRows, ModelMenu } from "../../ui/ModelMenu";
import {
	type Outcome,
	OutcomeNote,
	pulseOn,
	useOutcome,
} from "../../ui/Outcome";
import { Popover } from "../../ui/Overlay";
import { PermissionLine } from "../../ui/Permission";
import { Row, Rows } from "../../ui/Rows";
import { Segmented } from "../../ui/Segmented";
import { Switch } from "../../ui/Switch";

// The seven sections. Every control sends its intent; the engine's next view shows the result,
// and a form's refusal or success is its outcome.

// A settings section inside the Settings pane: its title, an optional line under it, then its
// controls. Not a screen (ui/Page is a screen's own panel, with back and tabs).
function Section({
	title,
	lede,
	action,
	children,
}: {
	title: string;
	lede?: string;
	action?: ReactNode;
	children: ReactNode;
}) {
	return (
		<div className="settings-page">
			<div className="settings-head">
				<h1 className="h1 grow">{title}</h1>
				{action}
			</div>
			{lede && <p className="sm muted settings-lede">{lede}</p>}
			{children}
		</div>
	);
}

export function Accounts({ section }: { section: AccountsSettings }) {
	const send = useSend();
	return (
		<Section title="Accounts">
			<Rows label="Accounts" className="settings-stack-gap settings-accounts">
				{section.accounts.map((account, i) => (
					<Row key={account.id} index={i} className="tile settings-account">
						<Account account={account} />
					</Row>
				))}
			</Rows>
			<Button
				variant="soft"
				size="lg"
				className="settings-start"
				onClick={() => send({ case: "openConnect", value: {} })}
			>
				Add account
			</Button>
		</Section>
	);
}

function Account({ account }: { account: SettingsAccount }) {
	const { tone, label } = accountStatus(account.state);
	const outcome = useOutcome({ case: "accountId", value: account.id });
	return (
		<>
			<span className="settings-account-head">
				<LogoTile
					brand={{ kind: "mail", provider: account.provider }}
					size={22}
					box={36}
				/>
				<span className="two-line grow">
					<span className="strong">{account.name}</span>
					<span className="xs subtle status-line">
						<Dot tone={tone} label={label} />
						{account.statusLine}
					</span>
				</span>
				<Manage account={account} />
			</span>
			<Signature account={account} />
			<OutcomeNote outcome={outcome} />
		</>
	);
}

// Reconnect (an expired sign-in only) and Disconnect, which asks first.
function Manage({ account }: { account: SettingsAccount }) {
	const send = useSend();
	const accountId = account.id;
	return (
		<Popover
			label={`Manage ${account.name}`}
			align="end"
			className="settings-manage"
			trigger={(props) => (
				<Button {...props} variant="ghost">
					Manage
				</Button>
			)}
		>
			{(dismiss) => (
				<>
					{match(
						(account.state ?? unset("SettingsAccount.state")).state,
						"AccountState.state",
						{
							synced: () => false,
							syncing: () => false,
							signInExpired: () => true,
						},
					) && (
						<button
							type="button"
							role="menuitem"
							className="menu-row press"
							onClick={() => {
								dismiss();
								send({ case: "reconnectAccount", value: { accountId } });
							}}
						>
							Sign in again
						</button>
					)}
					<Confirm
						question={`Disconnect ${account.name}? Its mail leaves EveryMail`}
						action="Disconnect"
						onConfirm={() => {
							dismiss();
							send({ case: "disconnectAccount", value: { accountId } });
						}}
						trigger={(props) => (
							<button
								type="button"
								{...props}
								role="menuitem"
								className="menu-row press bad"
							>
								Disconnect
							</button>
						)}
					/>
				</>
			)}
		</Popover>
	);
}

// The signature as typed; Save sends it, Remove clears it.
function Signature({ account }: { account: SettingsAccount }) {
	const send = useSend();
	const { signature } = account;
	const saved = match(signature, "SettingsAccount.signature", {
		text: (text) => text,
		none: () => "",
	});
	const [text, setText] = useDraft(saved);
	const accountId = account.id;
	return (
		<span className="settings-signature-edit">
			<Field
				label="Signature"
				hideLabel
				rows={2}
				placeholder="No signature"
				value={text}
				onChange={setText}
			/>
			<span className="settings-signature-actions">
				<Button
					variant="soft"
					disabled={text === saved}
					onClick={() =>
						send({
							case: "setSignature",
							value:
								text === ""
									? { accountId, signature: { case: "none", value: {} } }
									: { accountId, signature: { case: "text", value: text } },
						})
					}
				>
					Save signature
				</Button>
			</span>
		</span>
	);
}

export function AgentPermissions({
	section,
}: {
	section: AgentPermissionsSettings;
}) {
	const send = useSend();
	return (
		<Section
			title="Agent permissions"
			lede="Defaults for new agents. Each agent can change its own"
		>
			<div className="settings-list">
				{section.defaults.map((permission) => (
					<PermissionLine
						key={permission.id}
						permission={permission}
						onLevel={(level) =>
							send({
								case: "setDefaultPermission",
								value: { permissionId: permission.id, level },
							})
						}
					/>
				))}
			</div>
			<div className="settings-list">
				<SwitchLine
					label="Notify me when something waits"
					hint="Push to phone and desktop"
					on={section.notifyWhenWaiting}
					onToggle={() =>
						send({
							case: "setNotifyWhenWaiting",
							value: { notify: !section.notifyWhenWaiting },
						})
					}
				/>
				<SwitchLine
					label="Pause all agents"
					hint="Stops every schedule until you resume"
					on={section.agentsPaused}
					onToggle={() =>
						send({
							case: "setAgentsPaused",
							value: { paused: !section.agentsPaused },
						})
					}
				/>
			</div>
		</Section>
	);
}

export function Rules({ section }: { section: RulesSettings }) {
	const send = useSend();
	const { isOpen: open, toggle } = useDisclosure({
		case: "form",
		value: Form.NEW_RULE,
	}).of(section.newRule, "RulesSettings.new_rule");
	return (
		<Section
			title="Rules"
			lede="Fixed checks that run on every new mail, before any agent. Anything that needs judgement belongs to an agent"
			action={
				<Button variant="soft" aria-expanded={open} onClick={toggle}>
					<Icon name="plus" />
					New rule
				</Button>
			}
		>
			{open && <NewRule key={section.newRuleRevision} section={section} />}
			<Rows label="Rules" className="settings-stack-gap">
				{section.rules.map((rule, i) => (
					<Row key={rule.id} index={i} className="tile settings-rule">
						<span className="grow">{rule.text}</span>
						<Switch
							on={rule.enabled}
							label={`Rule active: ${rule.text}`}
							onToggle={() =>
								send({
									case: "setRuleEnabled",
									value: { ruleId: rule.id, enabled: !rule.enabled },
								})
							}
						/>
						<Confirm
							question="Remove this rule?"
							action="Remove"
							align="end"
							onConfirm={() =>
								send({ case: "removeRule", value: { ruleId: rule.id } })
							}
							trigger={(props) => (
								<IconButton {...props} icon="trash" label="Remove rule" />
							)}
						/>
					</Row>
				))}
			</Rows>
		</Section>
	);
}

// A choice's identity within its list: its case, and the folder or agent it names.
const conditionKey = (choice: RuleConditionChoice) =>
	choice.condition?.condition.case ?? unset("RuleConditionChoice.condition");
const actionKey = (choice: RuleActionChoice) => {
	const action = choice.action?.action ?? unset("RuleActionChoice.action");
	return typeof action.value === "string"
		? `${action.case}:${action.value}`
		: (action.case ?? unset("RuleAction.action"));
};

// The new rule, built from the engine's choices: one condition (with its text, when it takes one)
// and one or more actions. Added, the engine closes the form; refused, the reason shows under it.
// Picks and text are input before Add rule; the form starts over at each revision.
function NewRule({ section }: { section: RulesSettings }) {
	const send = useSend();
	const first = section.conditions[0] ?? unset("RulesSettings.conditions");
	const [picked, setPicked] = useDraft<string>(conditionKey(first));
	const [text, setText] = useDraft();
	const [actions, setActions] = useDraft<string[]>([]);
	const outcome = useOutcome({ case: "form", value: Form.NEW_RULE });
	const whenId = useId();
	const condition =
		section.conditions.find((choice) => conditionKey(choice) === picked) ??
		first;
	const add = () =>
		send({
			case: "addRule",
			value: {
				condition: filled(condition, text),
				actions: section.actions
					.filter((choice) => actions.includes(actionKey(choice)))
					.map((choice) => choice.action ?? unset("RuleActionChoice.action")),
			},
		});
	return (
		<div className="tile settings-new-rule reveal">
			<span className="settings-rule-fields">
				<span className="field">
					<label className="label" htmlFor={whenId}>
						When
					</label>
					<select
						id={whenId}
						value={picked}
						onChange={(event) => setPicked(event.target.value)}
					>
						{section.conditions.map((choice) => (
							<option key={conditionKey(choice)} value={conditionKey(choice)}>
								{choice.label}
							</option>
						))}
					</select>
				</span>
				{match(condition.parameter, "RuleConditionChoice.parameter", {
					none: () => null,
					text: (parameter) => (
						<Field
							label={condition.label}
							placeholder={parameter.placeholder}
							value={text}
							onChange={setText}
							onEnter={add}
						/>
					),
				})}
			</span>
			<span className="field">
				<span className="label">Then</span>
				<span className="form-chips">
					{section.actions.map((choice) => {
						const key = actionKey(choice);
						const on = actions.includes(key);
						return (
							<button
								key={key}
								type="button"
								className={cx("chip press", on && "on")}
								aria-pressed={on}
								onClick={() =>
									setActions(
										on
											? actions.filter((each) => each !== key)
											: [...actions, key],
									)
								}
							>
								{choice.label}
							</button>
						);
					})}
				</span>
			</span>
			<span className="settings-rule-foot">
				<OutcomeNote outcome={outcome} />
				<span className="grow" />
				<Button variant="primary" className={pulseOn(outcome)} onClick={add}>
					Add rule
				</Button>
			</span>
		</div>
	);
}

// The condition Add rule sends: the choice's own, with the typed text as its parameter. A words
// condition takes the text as one phrase: the field is one line, and the engine owns any split.
function filled(choice: RuleConditionChoice, text: string): RuleCondition {
	const condition = choice.condition ?? unset("RuleConditionChoice.condition");
	switch (condition.condition.case) {
		case "senderContains":
		case "senderDomain":
			return create(RuleConditionSchema, {
				condition: { case: condition.condition.case, value: text },
			});
		case "subjectContains":
			return create(RuleConditionSchema, {
				condition: { case: "subjectContains", value: { words: [text] } },
			});
		case "newsletter":
		case "newSenderWithLink":
			return condition;
		case undefined:
			return unset("RuleCondition.condition");
	}
}

export function BlockList({ section }: { section: BlockListSettings }) {
	return (
		<Section
			title="Block list"
			lede="Mail from these senders goes straight to Quarantine, without any agent running"
		>
			<BlockState state={section.state} />
		</Section>
	);
}

function BlockState({ state }: { state: BlockListSettings["state"] }) {
	switch (state.case) {
		case "blocked":
			return (
				<Rows label="Blocked senders" className="settings-blocked">
					{state.value.senders.map((sender, i) => (
						<Row
							key={sender.address}
							index={i}
							className="menu-row settings-blocked-row"
						>
							<span className="two-line grow">
								<span className="trunc">{sender.address}</span>
								<span className="xs subtle">{sender.blockedBy}</span>
							</span>
							<UnblockButton address={sender.address} />
						</Row>
					))}
				</Rows>
			);
		case "nobody":
			return <span className="settle sm subtle">Nobody is blocked</span>;
		case undefined:
			return unset("BlockListSettings.state");
	}
}

function UnblockButton({ address }: { address: string }) {
	const send = useSend();
	return (
		<Button
			variant="ghost"
			onClick={() => send({ case: "unblockSender", value: { address } })}
		>
			Remove
		</Button>
	);
}

export function Models({ section }: { section: ModelsSettings }) {
	return (
		<Section
			title="Models and keys"
			lede="EveryMail uses your own keys. A key is stored encrypted and never shown again"
		>
			<NoKeyNotice keys={section.keys} />
			<Rows label="Provider keys" className="settings-stack-gap">
				{section.providers.map((provider, i) => (
					<Row key={provider.provider} index={i} className="tile settings-key">
						<KeyRow provider={provider} />
					</Row>
				))}
			</Rows>
			<div className="settings-group">
				<span className="label">Model for each task</span>
				{section.tasks.map((task) => (
					<TaskModelMenu key={task.id} task={task} />
				))}
			</div>
			<Budget budget={section.budget} revision={section.budgetRevision} />
		</Section>
	);
}

function NoKeyNotice({ keys }: { keys: ModelsSettings["keys"] }) {
	switch (keys.case) {
		case "none":
			return (
				<div className="well settle settings-notice">
					<span className="strong">No key yet</span>
					<span className="sm muted">
						AI compose, Scribe and your own agents are off until you add one.
						Gatekeeper and Security Guard keep running their built-in checks
					</span>
				</div>
			);
		case "some":
			return null;
		case undefined:
			return unset("ModelsSettings.keys");
	}
}

// A key is pasted into a field while it is missing, or while the user replaces it (an engine
// disclosure); the engine empties the field by a new revision once it saved the key, which is
// never shown again. A refused key stays in the field with the reason under it.
function KeyRow({ provider }: { provider: ProviderKey }) {
	const send = useSend();
	const replace = useDisclosure({ case: "key", value: provider.provider });
	const brand = { kind: "model" as const, provider: provider.provider };
	const outcome = useOutcome({ case: "key", value: provider.provider });
	const field = (
		<KeyField key={provider.revision} provider={provider} outcome={outcome} />
	);
	return (
		<>
			<Logo brand={brand} size={22} className="settings-key-logo" />
			{match(provider.state, "ProviderKey.state", {
				missing: () => field,
				replacing: () => (
					<>
						{field}
						<Button variant="ghost" onClick={replace.close}>
							Cancel
						</Button>
					</>
				),
				set: () => (
					<>
						<span className="two-line grow">
							<span>{provider.name}</span>
							<span className={cx("xs ok", pulseOn(outcome))}>Key set</span>
							<OutcomeNote outcome={outcome} />
						</span>
						<Button variant="ghost" onClick={replace.open}>
							Replace
						</Button>
						<Confirm
							question={`Remove the ${provider.name} key?`}
							action="Remove"
							align="end"
							onConfirm={() =>
								send({
									case: "removeModelKey",
									value: { provider: provider.provider },
								})
							}
							trigger={(props) => (
								<Button {...props} variant="ghost">
									Remove
								</Button>
							)}
						/>
					</>
				),
			})}
		</>
	);
}

function KeyField({
	provider,
	outcome,
}: {
	provider: ProviderKey;
	outcome: Outcome;
}) {
	const send = useSend();
	const [key, setKey] = useDraft();
	const save = () =>
		send({ case: "setModelKey", value: { provider: provider.provider, key } });
	return (
		<>
			<span className="two-line grow">
				<span>{provider.name}</span>
				<Field
					label={provider.keyLabel}
					hideLabel
					type="password"
					placeholder="Paste a key"
					value={key}
					onChange={setKey}
					onEnter={save}
					className="settings-key-field"
				/>
				<OutcomeNote outcome={outcome} />
			</span>
			<Button variant="soft" className="settings-end" onClick={save}>
				Save
			</Button>
		</>
	);
}

function TaskModelMenu({ task }: { task: TaskModel }) {
	const send = useSend();
	return (
		<ModelMenu
			label={`Model for ${task.task}`}
			align="stretch"
			className="settings-task-menu"
			rows={choiceRows(task.choices)}
			onPick={(modelName) =>
				send({ case: "setTaskModel", value: { taskId: task.id, modelName } })
			}
			trigger={(props) => (
				<button
					type="button"
					{...props}
					className="menu-row press settings-task"
				>
					<span className="two-line grow">
						<span>{task.task}</span>
						<span className="xs subtle">{task.hint}</span>
					</span>
					<ModelChip model={task.model} />
					<Icon name="chevron-down" className="sm subtle" />
				</button>
			)}
		/>
	);
}

// The budget switch: off is unlimited; turning it on opens the field (an engine disclosure), and
// turning it off sets no budget, or closes the field when none was set. The field shows the saved
// budget and starts over at each revision; Set sends it as typed, and a refusal shows under it.
function Budget({
	budget,
	revision,
}: {
	budget: ModelsSettings["budget"];
	revision: number;
}) {
	const send = useSend();
	const entering = useDisclosure({ case: "form", value: Form.BUDGET });
	const shown = match(budget, "ModelsSettings.budget", {
		unlimited: () => ({
			on: false,
			toggle: entering.open,
			tokens: "",
			used: "",
		}),
		entering: () => ({
			on: true,
			toggle: entering.close,
			tokens: "",
			used: "",
		}),
		daily: (daily) => ({
			on: true,
			toggle: () =>
				send({
					case: "setBudget",
					value: { budget: { case: "unlimited", value: {} } },
				}),
			tokens: daily.tokens,
			used: daily.used,
		}),
	});
	return (
		<div className="settings-group">
			<SwitchLine
				label="Daily token budget"
				hint="Optional. At the budget new runs wait until tomorrow; sync and sending keep working"
				on={shown.on}
				onToggle={shown.toggle}
			/>
			{shown.on && (
				<DailyBudget key={revision} tokens={shown.tokens} used={shown.used} />
			)}
		</div>
	);
}

function DailyBudget({ tokens, used }: { tokens: string; used: string }) {
	const send = useSend();
	const [value, setValue] = useDraft(tokens);
	const outcome = useOutcome({ case: "form", value: Form.BUDGET });
	const set = () =>
		send({ case: "setBudget", value: { budget: { case: "daily", value } } });
	return (
		<span className="reveal settings-budget">
			<Field
				label="Tokens per day"
				hideLabel
				value={value}
				onChange={setValue}
				onEnter={set}
				className="settings-budget-field"
			/>
			<span className="sm subtle">tokens a day</span>
			<Button variant="soft" className={pulseOn(outcome)} onClick={set}>
				Set
			</Button>
			<span className="sm subtle">{used}</span>
			<OutcomeNote outcome={outcome} />
		</span>
	);
}

export function Appearance({ section }: { section: AppearanceSettings }) {
	const send = useSend();
	return (
		<Section title="Appearance">
			<div className="settings-group">
				<span className="label">Theme</span>
				<Segmented
					label="Theme"
					kind="radio"
					className="settings-appearance-seg"
					segments={section.themes.map((choice) => ({
						key: String(choice.theme),
						label: choice.label,
						selected: choice.selected,
						onSelect: () =>
							send({ case: "setTheme", value: { theme: choice.theme } }),
					}))}
				/>
			</div>
			<div className="settings-group">
				<span className="label">List density</span>
				<Segmented
					label="Density"
					kind="radio"
					className="settings-appearance-seg"
					segments={section.densities.map((choice) => ({
						key: choice.label,
						label: choice.label,
						selected: choice.selected,
						onSelect: () =>
							send({
								case: "setDensity",
								value: { density: choice.density },
							}),
					}))}
				/>
			</div>
		</Section>
	);
}

export function YourData({ section }: { section: YourDataSettings }) {
	const send = useSend();
	return (
		<Section title="Your data">
			<SettingLine
				label={<span className="strong">Export</span>}
				hintClass="sm muted"
				hint="One file with what only EveryMail holds: your agents and their instructions, notes, rules, block list and run log. Your mail stays with your provider"
			>
				<Export section={section} />
			</SettingLine>
			<SettingLine
				label={<span className="strong">Sign out</span>}
				hintClass="sm muted"
				hint="Back to sign-in. What is on this device stays until someone else signs in"
			>
				<Button
					variant="soft"
					onClick={() => send({ case: "signOut", value: {} })}
				>
					Sign out
				</Button>
			</SettingLine>
			<SettingLine
				label={<span className="strong">Reset demo data</span>}
				hintClass="sm muted"
				hint="Drops what is on this device and fills it again from the start"
			>
				<Confirm
					question="Reset all the data on this device?"
					action="Reset"
					align="end"
					onConfirm={() => send({ case: "resetLocalData", value: {} })}
					trigger={(props) => (
						<Button {...props} variant="ghost">
							Reset
						</Button>
					)}
				/>
			</SettingLine>
			<SettingLine
				label={<span className="strong">Delete account</span>}
				hintClass="sm muted"
				hint="Disconnects every mailbox and deletes everything EveryMail holds, at once. Nothing changes at your providers"
			>
				<Confirm
					question="Delete your EveryMail account and everything in it?"
					action="Delete account"
					align="end"
					onConfirm={() => send({ case: "deleteUserAccount", value: {} })}
					trigger={(props) => (
						<Button {...props} variant="ghost" className="bad">
							Delete account
						</Button>
					)}
				/>
			</SettingLine>
		</Section>
	);
}

// Export, its progress while the file is prepared, then the file.
function Export({ section }: { section: YourDataSettings }) {
	const send = useSend();
	const { export: state } = section;
	const start = () => send({ case: "exportData", value: {} });
	switch (state.case) {
		case "idle":
			return (
				<Button variant="soft" className="settings-export" onClick={start}>
					Export
				</Button>
			);
		case "running":
			return (
				<span className="settings-export-state settle">
					<span className="xs subtle">{state.value.label}</span>
					<span
						className="bar settings-export-bar"
						role="progressbar"
						aria-label={state.value.label}
						aria-valuenow={state.value.percent}
						aria-valuemin={0}
						aria-valuemax={100}
					>
						<i style={{ width: `${state.value.percent}%` }} />
					</span>
					<button
						type="button"
						className="link xs"
						onClick={() => send({ case: "cancelExport", value: {} })}
					>
						Cancel
					</button>
				</span>
			);
		case "failed":
			return (
				<span className="settings-export-state settle">
					<span className="xs bad" role="alert">
						{state.value.reason}
					</span>
					<button type="button" className="link xs" onClick={start}>
						Try again
					</button>
				</span>
			);
		case "ready":
			return (
				<span className="settings-export-state settle">
					<span className="xs">{state.value.text}</span>
					<span className="xs subtle">{state.value.file}</span>
					<button type="button" className="link xs" onClick={start}>
						Export again
					</button>
				</span>
			);
		case undefined:
			return unset("YourDataSettings.export");
	}
}
