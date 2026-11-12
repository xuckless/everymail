import type { ReactNode } from "react";
import { unset, useSend } from "../../engine";
import type {
	AgentSettings,
	SystemInstructions,
} from "../../gen/ui/v1/agents_pb";
import { Form } from "../../gen/ui/v1/common_pb";
import { useDebounce, useDraft } from "../../presentation";
import { Button, IconButton } from "../../ui/Button";
import { Confirm } from "../../ui/Confirm";
import { Field } from "../../ui/Field";
import { Icon } from "../../ui/Icon";
import { SettingLine } from "../../ui/Lines";
import { Logo, ModelChip } from "../../ui/Logo";
import { choiceRows, ModelMenu } from "../../ui/ModelMenu";
import { OutcomeNote, useOutcome } from "../../ui/Outcome";
import { Popover } from "../../ui/Overlay";
import { PermissionLine } from "../../ui/Permission";
import { Switch } from "../../ui/Switch";

// What configures the agent, as a settings form: a label column on the left and its controls on
// the right where there is room. Instructions are sent on a debounce; every other control sends
// its intent at once and the engine's view shows the result. A refused permission level (Auto on
// a Gmail Send) comes back as the agent's outcome, shown under the permissions. The agent's one
// active switch is on its page's header (Agents.tsx).
export function Configure({
	settings,
	agentId,
}: {
	settings: AgentSettings;
	agentId: string;
}) {
	const send = useSend();
	const outcome = useOutcome({ case: "agentId", value: agentId });
	return (
		<div className="form">
			<FormSection title="Instructions">
				<Instructions instructions={settings.instructions} agentId={agentId} />
			</FormSection>
			<FormSection title="When it runs">
				<Schedule settings={settings} agentId={agentId} />
			</FormSection>
			<FormSection title="Model" hint={settings.keyNote}>
				<AgentModelMenu settings={settings} agentId={agentId} />
			</FormSection>
			<FormSection title="What it may do">
				<div className="settings-list">
					{settings.permissions.map((permission) => (
						<PermissionLine
							key={permission.id}
							permission={permission}
							onLevel={(level) =>
								send({
									case: "setAgentPermission",
									value: { agentId, permissionId: permission.id, level },
								})
							}
						/>
					))}
				</div>
				<OutcomeNote outcome={outcome} />
			</FormSection>
			<FormSection title="Accounts it can see">
				<div className="settings-list">
					{settings.accounts.map((account) => (
						<SettingLine
							key={account.id}
							label={
								<span className="status-line">
									<Logo brand={{ kind: "mail", provider: account.provider }} />
									{account.name}
								</span>
							}
						>
							<Switch
								on={account.included}
								label={account.name}
								onToggle={() =>
									send({
										case: "setAgentAccount",
										value: {
											agentId,
											accountId: account.id,
											included: !account.included,
										},
									})
								}
							/>
						</SettingLine>
					))}
				</div>
			</FormSection>
			<News news={settings.news} />
			<FormSection title="Delete">
				<Removal removal={settings.removal} agentId={agentId} />
			</FormSection>
		</div>
	);
}

function FormSection({
	title,
	hint,
	children,
}: {
	title: string;
	hint?: string;
	children: ReactNode;
}) {
	return (
		<section className="form-section">
			<span className="form-label">
				<h2 className="strong sm">{title}</h2>
				{hint && <span className="xs subtle">{hint}</span>}
			</span>
			<div className="form-body">{children}</div>
		</section>
	);
}

function Instructions({
	instructions,
	agentId,
}: {
	instructions: AgentSettings["instructions"];
	agentId: string;
}) {
	switch (instructions.case) {
		case "system":
			return <SystemText system={instructions.value} agentId={agentId} />;
		case "user":
			return (
				<InstructionsField
					agentId={agentId}
					initial={instructions.value.text}
					label="Instructions"
					hideLabel
					rows={6}
				/>
			);
		case undefined:
			return unset("AgentSettings.instructions");
	}
}

function SystemText({
	system,
	agentId,
}: {
	system: SystemInstructions;
	agentId: string;
}) {
	return (
		<>
			<span className="xs subtle status-line">
				<Icon name="lock" size="sm" />
				Built in, kept up to date by EveryMail
			</span>
			<div className="well sm muted form-text">{system.builtIn}</div>
			<InstructionsField
				agentId={agentId}
				initial={system.yours}
				label={system.yoursLabel}
				rows={3}
				placeholder="Added after the built-in ones"
			/>
		</>
	);
}

// Typed text is input; it reaches the engine on a debounce.
function InstructionsField({
	agentId,
	initial,
	...field
}: {
	agentId: string;
	initial: string;
	label: string;
	rows: number;
	hideLabel?: boolean;
	placeholder?: string;
}) {
	const send = useSend();
	const [text, setText] = useDraft(initial);
	const { schedule } = useDebounce();
	return (
		<Field
			{...field}
			value={text}
			onChange={(next) => {
				setText(next);
				schedule(() =>
					send({
						case: "setAgentInstructions",
						value: { agentId, text: next },
					}),
				);
			}}
		/>
	);
}

function Schedule({
	settings,
	agentId,
}: {
	settings: AgentSettings;
	agentId: string;
}) {
	const send = useSend();
	const { schedule } = settings;
	switch (schedule.case) {
		case "onEvent":
		case "onMessage":
		case "every":
			return <span className="sm">{schedule.value.when}</span>;
		case "atTimes":
			return (
				<span className="form-chips">
					<span className="sm">{schedule.value.when}</span>
					{schedule.value.setTimes.map((time) => (
						<span key={time.minute} className="chip form-chip settle">
							{time.label}
							<IconButton
								icon="close"
								size="sm"
								label={`Remove ${time.label}`}
								onClick={() =>
									send({
										case: "removeAgentTime",
										value: { agentId, minute: time.minute },
									})
								}
							/>
						</span>
					))}
					<Popover
						label="Add a time"
						className="form-times"
						trigger={(props) => (
							<button type="button" {...props} className="chip press">
								<Icon name="plus" size="sm" />
								Add a time
							</button>
						)}
					>
						{(dismiss) =>
							settings.timeChoices.map((choice) => (
								<button
									key={choice.minute}
									type="button"
									role="menuitem"
									className="menu-row press"
									onClick={() => {
										dismiss();
										send({
											case: "addAgentTime",
											value: { agentId, minute: choice.minute },
										});
									}}
								>
									{choice.label}
								</button>
							))
						}
					</Popover>
				</span>
			);
		case undefined:
			return unset("AgentSettings.schedule");
	}
}

// The agent's model; a model whose provider has no key leads to the Models settings.
function AgentModelMenu({
	settings,
	agentId,
}: {
	settings: AgentSettings;
	agentId: string;
}) {
	const send = useSend();
	return (
		<ModelMenu
			label="Model"
			className="model-menu"
			rows={choiceRows(settings.models)}
			onPick={(modelName) =>
				send({ case: "setAgentModel", value: { agentId, modelName } })
			}
			trigger={(props) => (
				<button type="button" {...props} className="menu-row press form-model">
					<ModelChip model={settings.model} />
					<span className="grow" />
					<Icon name="chevron-down" className="subtle" />
				</button>
			)}
		/>
	);
}

function News({ news }: { news: AgentSettings["news"] }) {
	switch (news.case) {
		case "topics":
			return (
				<FormSection title="Topics" hint={news.value.cap}>
					<Topics topics={news.value.topics} revision={news.value.revision} />
				</FormSection>
			);
		case "none":
			return null;
		case undefined:
			return unset("AgentSettings.news");
	}
}

// The topics and the field that adds one. The engine empties the field by a new revision once a
// topic was added; a refused one stays, with the reason under it.
function Topics({ topics, revision }: { topics: string[]; revision: number }) {
	const send = useSend();
	const outcome = useOutcome({ case: "form", value: Form.NEWS_TOPIC });
	return (
		<>
			<span className="form-chips">
				{topics.map((each) => (
					<span key={each} className="chip form-chip settle">
						{each}
						<IconButton
							icon="close"
							size="sm"
							label={`Remove ${each}`}
							onClick={() =>
								send({ case: "removeNewsTopic", value: { topic: each } })
							}
						/>
					</span>
				))}
			</span>
			<AddTopic
				key={revision}
				onAdd={(topic) => send({ case: "addNewsTopic", value: { topic } })}
			/>
			<OutcomeNote outcome={outcome} />
		</>
	);
}

function AddTopic({ onAdd }: { onAdd: (topic: string) => void }) {
	const [topic, setTopic] = useDraft();
	return (
		<span className="form-add">
			<Field
				label="Add a topic"
				hideLabel
				placeholder="Add a topic"
				value={topic}
				onChange={setTopic}
				onEnter={() => onAdd(topic)}
			/>
			<Button variant="soft" onClick={() => onAdd(topic)}>
				Add
			</Button>
		</span>
	);
}

function Removal({
	removal,
	agentId,
}: {
	removal: AgentSettings["removal"];
	agentId: string;
}) {
	const send = useSend();
	switch (removal.case) {
		case "delete":
			return (
				<Confirm
					question="Delete this agent and its history?"
					action="Delete"
					onConfirm={() => send({ case: "removeAgent", value: { agentId } })}
					trigger={(props) => (
						<Button {...props} variant="ghost" className="form-delete bad">
							{removal.value.label}
						</Button>
					)}
				/>
			);
		case "deactivateOnly":
			return <span className="xs subtle">{removal.value.text}</span>;
		case undefined:
			return unset("AgentSettings.removal");
	}
}
