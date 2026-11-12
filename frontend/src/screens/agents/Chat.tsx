import { modelSettings } from "../../chrome/places";
import { match, unset, useSend } from "../../engine";
import type { AgentChat, ChatTurn } from "../../gen/ui/v1/agents_pb";
import { Button } from "../../ui/Button";
import { Confirm } from "../../ui/Confirm";
import { Dot } from "../../ui/Dot";
import { MessageBox } from "../../ui/MessageBox";
import { Paragraphs } from "../../ui/Paragraphs";
import { References } from "../../ui/Reference";
import { Row, Rows } from "../../ui/Rows";

// Writing to an agent: the turns, oldest first, and the message box the engine's availability
// allows. Send and ⌘Enter send the message; the box clears when the engine says it took it.

// The conversation over the composer. The scroll area is column-reverse, so it starts at the newest
// turn and stays there as turns arrive, without the Frontend holding a scroll position.
export function Chat({ chat, agentId }: { chat: AgentChat; agentId: string }) {
	const send = useSend();
	return (
		<div className="chat">
			<div className="scroll chat-scroll">
				<div className="chat-inner">
					<Conversation conversation={chat.conversation} />
				</div>
			</div>
			<div className="chat-foot">
				{match(chat.conversation, "AgentChat.conversation", {
					empty: () => null,
					turns: () => (
						<Confirm
							question="Clear this conversation?"
							action="Clear"
							side="above"
							onConfirm={() =>
								send({ case: "clearConversation", value: { agentId } })
							}
							trigger={(props) => (
								<button type="button" {...props} className="link xs">
									Clear conversation
								</button>
							)}
						/>
					),
				})}
				<Availability
					availability={chat.availability}
					agentId={agentId}
					revision={chat.revision}
				/>
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
					<ApprovalLink title={state.value.approvalTitle} />
				</span>
			);
		case "answered":
			return (
				<div className="chat-answer">
					<Paragraphs texts={state.value.paragraphs} settle />
					<References
						references={state.value.references}
						className="sm settle"
					/>
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

// The approval the turn waits for is on the Approvals screen.
function ApprovalLink({ title }: { title: string }) {
	const send = useSend();
	return (
		<button
			type="button"
			className="link"
			onClick={() => send({ case: "openApprovals", value: {} })}
		>
			{title}
		</button>
	);
}

function Availability({
	availability,
	agentId,
	revision,
}: {
	availability: AgentChat["availability"];
	agentId: string;
	// AgentChat.revision: the box starts empty when the engine took a message.
	revision: number;
}) {
	const send = useSend();
	switch (availability.case) {
		case "ready":
			return (
				<MessageBox
					key={revision}
					label={availability.value.placeholder}
					placeholder={availability.value.placeholder}
					action="Send"
					onSubmit={(text) =>
						send({ case: "sendChatMessage", value: { agentId, text } })
					}
				/>
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
