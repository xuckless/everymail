import type { MouseEvent, ReactNode } from "react";
import { unset, useSend } from "../../engine";
import type { Model } from "../../gen/ui/v1/common_pb";
import type {
	AccountsSettings,
	AgentPermissionsSettings,
	AppearanceSettings,
	BlockListSettings,
	ModelsSettings,
	ProviderKey,
	RulesSettings,
	SettingsAccount,
} from "../../gen/ui/v1/settings_pb";
import { useDraft } from "../../presentation";
import { Button } from "../../ui/Button";
import { accountStatus, Dot } from "../../ui/Dot";
import { Field } from "../../ui/Field";
import { Icon } from "../../ui/Icon";
import { SettingLine, SwitchLine } from "../../ui/Lines";
import { Logo, LogoTile } from "../../ui/Logo";
import { PermissionLine } from "../../ui/Permission";
import { Row, Rows } from "../../ui/Rows";
import { Segmented } from "../../ui/Segmented";
import { Switch } from "../../ui/Switch";

// The seven sections. Controls without an intent in the contract have no handler.

function Page({
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
		<Page title="Accounts">
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
		</Page>
	);
}

function Account({ account }: { account: SettingsAccount }) {
	const { tone, label } = accountStatus(account.status);
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
				<Button variant="ghost">Manage</Button>
			</span>
			<Signature signature={account.signature} />
		</>
	);
}

function Signature({ signature }: { signature: SettingsAccount["signature"] }) {
	switch (signature.case) {
		case "text":
			return (
				<span className="well sm muted settings-signature">
					{signature.value}
				</span>
			);
		case "none":
			return (
				<span className="well sm muted settings-signature">No signature</span>
			);
		case undefined:
			return unset("SettingsAccount.signature");
	}
}

export function AgentPermissions({
	section,
}: {
	section: AgentPermissionsSettings;
}) {
	return (
		<Page
			title="Agent permissions"
			lede="Defaults for new agents. Each agent can change its own"
		>
			<div className="settings-list">
				{section.defaults.map((permission) => (
					<PermissionLine key={permission.action} permission={permission} />
				))}
			</div>
			<div className="settings-list">
				<SwitchLine
					label="Notify me when something waits"
					hint="Push to phone and desktop"
					on={section.notifyWhenWaiting}
				/>
				<SwitchLine
					label="Pause all agents"
					hint="Stops every schedule until you resume"
					on={section.agentsPaused}
				/>
			</div>
		</Page>
	);
}

export function Rules({ section }: { section: RulesSettings }) {
	return (
		<Page
			action={<Button variant="soft">New rule</Button>}
			title="Rules"
			lede="Fixed checks that run on every new mail, before any agent. Anything that needs judgement belongs to an agent"
		>
			<Rows label="Rules" className="settings-stack-gap">
				{section.rules.map((rule, i) => (
					<Row key={rule.id} index={i} className="tile settings-rule">
						<span className="two-line grow">
							<span>
								<span className="subtle">When </span>
								{rule.when}
							</span>
							<span>
								<span className="subtle">then </span>
								{rule.then}
							</span>
						</span>
						<Switch on={rule.enabled} label="Rule active" />
					</Row>
				))}
			</Rows>
		</Page>
	);
}

export function BlockList({ section }: { section: BlockListSettings }) {
	return (
		<Page
			title="Block list"
			lede="Mail from these senders goes straight to Quarantine, without any agent running"
		>
			<BlockState state={section.state} />
		</Page>
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
							<Button variant="ghost">Remove</Button>
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

export function Models({ section }: { section: ModelsSettings }) {
	return (
		<Page
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
					<button
						key={task.task}
						type="button"
						className="menu-row press settings-task"
					>
						<span className="two-line grow">
							<span>{task.task}</span>
							<span className="xs subtle">{task.hint}</span>
						</span>
						<ModelChip model={task.model} />
						<Icon name="chevron-right" className="sm subtle" />
					</button>
				))}
			</div>
			<Budget budget={section.budget} />
		</Page>
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

function ModelChip({ model }: { model: Model | undefined }) {
	if (!model) return unset("TaskModel.model");
	return (
		<>
			<Logo brand={{ kind: "model", provider: model.provider }} size={18} />
			<span className="sm">{model.name}</span>
		</>
	);
}

function KeyRow({ provider }: { provider: ProviderKey }) {
	const [key, setKey] = useDraft();
	const brand = { kind: "model" as const, provider: provider.provider };
	const state = provider.state;
	return (
		<>
			<Logo brand={brand} size={22} className="settings-key-logo" />
			<span className="two-line grow">
				<span>{provider.name}</span>
				{state.case === "set" && (
					<span className="xs settings-ok">Key set</span>
				)}
				{state.case === "missing" && (
					<Field
						label={provider.keyLabel}
						hideLabel
						type="password"
						placeholder="Paste a key"
						value={key}
						onChange={setKey}
						className="settings-key-field"
					/>
				)}
			</span>
			{state.case === "set" && (
				<>
					<Button variant="ghost">Replace</Button>
					<Button variant="ghost">Remove</Button>
				</>
			)}
			{state.case === "missing" && (
				<Button variant="soft" className="settings-end">
					Save
				</Button>
			)}
			{state.case === undefined && unset("ProviderKey.state")}
		</>
	);
}

function Budget({ budget }: { budget: ModelsSettings["budget"] }) {
	return (
		<div className="settings-group">
			<SwitchLine
				label="Daily token budget"
				hint="Optional. At the budget new runs wait until tomorrow; sync and sending keep working"
				on={budget.case === "daily"}
			/>
			<BudgetField budget={budget} />
		</div>
	);
}

function BudgetField({ budget }: { budget: ModelsSettings["budget"] }) {
	switch (budget.case) {
		case "unlimited":
			return null;
		case "daily":
			return (
				<DailyBudget tokens={budget.value.tokens} used={budget.value.used} />
			);
		case undefined:
			return unset("ModelsSettings.budget");
	}
}

function DailyBudget({ tokens, used }: { tokens: string; used: string }) {
	const [value, setValue] = useDraft(tokens);
	return (
		<span className="reveal settings-budget">
			<Field
				label="Tokens per day"
				hideLabel
				value={value}
				onChange={setValue}
				className="settings-budget-field"
			/>
			<span className="sm subtle">tokens a day</span>
			<span className="sm subtle">{used}</span>
		</span>
	);
}

// The press point starts the theme's circular reveal (shell.tsx reads --vx and --vy).
function revealFrom(event: MouseEvent) {
	const root = document.documentElement.style;
	root.setProperty("--vx", `${event.clientX}px`);
	root.setProperty("--vy", `${event.clientY}px`);
}

export function Appearance({ section }: { section: AppearanceSettings }) {
	const send = useSend();
	return (
		<Page title="Appearance">
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
						onSelect: (event) => {
							revealFrom(event);
							send({ case: "setTheme", value: { theme: choice.theme } });
						},
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
		</Page>
	);
}

export function YourData() {
	return (
		<Page title="Your data">
			<SettingLine
				label={<span className="strong">Export</span>}
				hintClass="sm muted"
				hint="One file with what only EveryMail holds: your agents and their instructions, notes, rules, block list and run log. Your mail stays with your provider"
			>
				<Button variant="soft" className="settings-export">
					Export
				</Button>
			</SettingLine>
			<SettingLine
				label={<span className="strong">Delete account</span>}
				hintClass="sm muted"
				hint="Disconnects every mailbox and deletes everything EveryMail holds, at once. Nothing changes at your providers"
			>
				<Button variant="ghost" className="settings-danger">
					Delete account
				</Button>
			</SettingLine>
		</Page>
	);
}
