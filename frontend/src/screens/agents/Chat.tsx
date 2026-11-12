import { modelSettings } from "../../chrome/places";
import { unset, useSend } from "../../engine";
import {
	type AgentChat,
	type AgentDetail,
	AgentDetailPane,
	AgentsPane,
	type AgentsView,
	type ChatReference,
	type ChatTurn,
} from "../../gen/ui/v1/agents_pb";
import { useLayout } from "../../presentation";
import { Button } from "../../ui/Button";
import { Composer } from "../../ui/Composer";
import { Dot } from "../../ui/Dot";
import { Paragraphs } from "../../ui/Paragraphs";
import { Row, Rows } from "../../ui/Rows";
import { Segmented } from "../../ui/Segmented";
import { approvalElement } from "./Queue";

// Writing to an agent: the turns, oldest first, and the composer the engine's availability allows.
// Sending has no intent yet, so Send is drawn without one.

// The selected agent's chat, when it has one.
export function chatOf(detail: AgentsView["detail"]): AgentChat | undefined {
	switch (detail.case) {
		case "agent":
			return agentChat(detail.value.chat);
		case "none":
			return undefined;
		case undefined:
			return unset("AgentsView.detail");
	}
}

export function agentChat(chat: AgentDetail["chat"]): AgentChat | undefined {
	switch (chat.case) {
		case "available":
			return chat.value;
		case "none":
			return undefined;
		case undefined:
			return unset("AgentDetail.chat");
	}
}

export function currentDetailPane(chat: AgentChat) {
	return (
		chat.panes.find((tab) => tab.current)?.pane ?? unset("AgentChat.panes")
	);
}

// The phone shows the chat over the whole screen while it is the current pane.
export function chatCoversPhone(view: AgentsView) {
	const chat = chatOf(view.detail);
	return chat !== undefined && currentDetailPane(chat) === AgentDetailPane.CHAT;
}

export function DetailPanes({ chat }: { chat: AgentChat }) {
	const send = useSend();
	return (
		<Segmented
			kind="tabs"
			label="Agent"
			className="detail-panes"
			segments={chat.panes.map((tab) => ({
				key: String(tab.pane),
				label: tab.label,
				selected: tab.current,
				onSelect: () =>
					send({ case: "selectAgentDetailPane", value: { pane: tab.pane } }),
			}))}
		/>
	);
}

// The conversation over the composer. The scroll area is column-reverse, so it starts at the newest
// turn and stays there as turns arrive, without the Frontend holding a scroll position.
export function Chat({ chat }: { chat: AgentChat }) {
	return (
		<div className="chat">
			<div className="scroll chat-scroll">
				<div className="chat-inner">
					<Conversation conversation={chat.conversation} />
				</div>
			</div>
			<div className="chat-foot">
				<Availability availability={chat.availability} />
			</div>
		</div>
	);
}

function Conversation({
	conversation,
}: {
	conversation: AgentChat["conversation"];
}) {
	switch (conversation.case) {
		case "turns":
			return (
				<Rows label="Conversation" className="chat-turns">
					{conversation.value.turns.map((turn, i) => (
						<Row key={turn.id} index={i}>
							<Turn turn={turn} />
						</Row>
					))}
				</Rows>
			);
		case "empty":
			return (
				<p className="sm subtle settle chat-empty">{conversation.value.hint}</p>
			);
		case undefined:
			return unset("AgentChat.conversation");
	}
}

function Turn({ turn }: { turn: ChatTurn }) {
	return (
		<article className="chat-turn">
			<div className="chat-asked">
				<span className="grow chat-message">{turn.message}</span>
				<span className="xs subtle">{turn.time}</span>
			</div>
			<TurnState state={turn.state} />
		</article>
	);
}

function TurnState({ state }: { state: ChatTurn["state"] }) {
	switch (state.case) {
		case "running":
			return (
				<span className="sm muted status-line settle">
					<Dot tone="run" />
					<span className="shimmer">{state.value.label}</span>
				</span>
			);
		case "waiting":
			return (
				<span className="sm status-line settle">
					<Dot tone="wait" />
					<span className="muted">{state.value.label}</span>
					<ApprovalLink
						approvalId={state.value.approvalId}
						title={state.value.approvalTitle}
					/>
				</span>
			);
		case "answered":
			return (
				<div className="chat-answer">
					<Paragraphs texts={state.value.paragraphs} settle />
					{state.value.references.length > 0 && (
						<span className="chat-references settle">
							{state.value.references.map((reference) => (
								<Reference key={reference.label} reference={reference} />
							))}
						</span>
					)}
				</div>
			);
		case "failed":
			return (
				<span className="sm status-line settle">
					<Dot tone="err" />
					<span className="muted">{state.value.reason}</span>
				</span>
			);
		case "stopped":
			return (
				<span className="sm status-line settle">
					<Dot tone="idle" />
					<span className="muted">{state.value.text}</span>
				</span>
			);
		case undefined:
			return unset("ChatTurn.state");
	}
}

// Narrow layouts show the queue as a pane: the link selects it. Wide layouts show the queue beside
// the chat: the link brings the approval into view there and focuses it.
function ApprovalLink({
	approvalId,
	title,
}: {
	approvalId: string;
	title: string;
}) {
	const send = useSend();
	const wide = useLayout() === "split";
	return (
		<button
			type="button"
			className="link"
			onClick={() => {
				if (!wide)
					return send({
						case: "selectAgentsPane",
						value: { pane: AgentsPane.APPROVALS },
					});
				const item = document.getElementById(approvalElement(approvalId));
				item?.scrollIntoView({ behavior: "smooth", block: "nearest" });
				item?.focus({ preventScroll: true });
			}}
		>
			{title}
		</button>
	);
}

function Reference({ reference }: { reference: ChatReference }) {
	const send = useSend();
	const { target } = reference;
	return (
		<button
			type="button"
			className="link sm chat-reference"
			onClick={() => {
				switch (target.case) {
					case "thread":
						return send({
							case: "openThread",
							value: { threadId: target.value.threadId },
						});
					case "event":
						return send({
							case: "selectEvent",
							value: { eventId: target.value.eventId },
						});
					case undefined:
						return unset("ChatReference.target");
				}
			}}
		>
			{reference.label}
		</button>
	);
}

function Availability({
	availability,
}: {
	availability: AgentChat["availability"];
}) {
	const send = useSend();
	switch (availability.case) {
		case "ready":
			return (
				<Composer
					label={availability.value.placeholder}
					placeholder={availability.value.placeholder}
				>
					<span className="grow" />
					<Button variant="primary">Send</Button>
				</Composer>
			);
		case "busy":
			return (
				<p className="sm muted chat-note settle">{availability.value.text}</p>
			);
		case "off":
			return (
				<div className="chat-note chat-off settle">
					<span className="sm muted grow">{availability.value.text}</span>
					<Button variant="soft" onClick={() => send(modelSettings)}>
						{availability.value.action}
					</Button>
				</div>
			);
		case "deactivated":
			return (
				<p className="sm muted chat-note settle">{availability.value.text}</p>
			);
		case undefined:
			return unset("AgentChat.availability");
	}
}
