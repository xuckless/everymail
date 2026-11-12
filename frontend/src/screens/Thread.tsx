import "./thread.css";
import { anyOf, modelProviders } from "@everymail/design/brands";
import { modelSettings } from "../chrome/places";
import { match, unset, useSend } from "../engine";
import type {
	ComposeReady,
	ReplyPanel,
	ThreadMessage,
	ThreadView,
} from "../gen/ui/v1/thread_pb";
import { useLayout, useOverlay } from "../presentation";
import { AgentDraft } from "../ui/AgentDraft";
import { Button, IconButton } from "../ui/Button";
import { cx } from "../ui/cx";
import { Icon } from "../ui/Icon";
import { ModelChip } from "../ui/Logo";
import { MailToolbar, MessageCard } from "../ui/MailCard";
import { MessageBody } from "../ui/MessageBody";
import { MessageBox } from "../ui/MessageBox";
import { choiceRows, ModelMenu } from "../ui/ModelMenu";
import { Overlap } from "../ui/Overlap";
import { UserSheet } from "../ui/Overlay";
import { Paragraphs } from "../ui/Paragraphs";
import { PhoneHead } from "../ui/PhoneHead";
import { Row, Rows } from "../ui/Rows";
import { Segmented } from "../ui/Segmented";
import { Summary } from "../ui/Summary";
import { stagger } from "../ui/stagger";
import type { ScreenProps } from "./Screen";

// One thread with Reply with AI beside it (desktop) or in a sheet (phone): the toolbar, the subject,
// Scribe's summary and a card per message. Back returns to the inbox. The toolbar carries the
// thread's actions and the latest message's star (ThreadView.message_id).
export function Thread({ view }: ScreenProps<ThreadView>) {
	const layout = useLayout();
	switch (layout) {
		case "phone":
			return <PhoneThread view={view} />;
		case "list":
			return <ThreadPanel view={view} />;
		case "split":
			return (
				<>
					<ThreadPanel view={view} />
					<ReplyAside reply={view.reply} threadId={view.threadId} />
				</>
			);
	}
}

const back = { case: "back", value: {} } as const;

function ThreadPanel({ view }: { view: ThreadView }) {
	const send = useSend();
	return (
		<section className="panel thread-main" aria-label={view.subject}>
			<MailToolbar
				threadId={view.threadId}
				actions={view.actions}
				place={view.place}
				star={{ messageId: view.messageId, starred: view.starred }}
				lead={
					<IconButton
						icon="chevron-left"
						label="Back to inbox"
						onClick={() => send(back)}
					/>
				}
			/>
			<div className="scroll thread-scroll">
				<ThreadContent view={view} />
			</div>
		</section>
	);
}

function ThreadContent({ view }: { view: ThreadView }) {
	return (
		<div className="thread-body">
			<header className="thread-head">
				<h1 className="h1 thread-subject">{view.subject}</h1>
				<span className="sm subtle">{view.context}</span>
			</header>
			<Summary summary={view.scribe} label="Thread summary by Scribe" />
			{view.messages.map((message) => (
				<Message key={message.messageId} message={message} />
			))}
			<Rows label="Agent drafts">
				{view.drafts.map((draft, i) => (
					<Row key={draft.approvalId} index={i}>
						<AgentDraft draft={draft} />
					</Row>
				))}
			</Rows>
		</div>
	);
}

// A message's card; its head expands and collapses it.
function Message({ message }: { message: ThreadMessage }) {
	const send = useSend();
	const { display, messageId } = message;
	const shown = match(display, "ThreadMessage.display", {
		collapsed: () => ({
			expanded: false,
			invitations: [],
			body: null,
			toggle: { case: "expandMessage", value: { messageId } } as const,
		}),
		expanded: (expanded) => ({
			expanded: true,
			invitations: expanded.invitations,
			body: <MessageBody url={expanded.bodyUrl} title={message.subject} />,
			toggle: { case: "collapseMessage", value: { messageId } } as const,
		}),
	});
	return (
		<MessageCard
			messageId={messageId}
			from={message.from}
			recipients={message.recipients}
			time={message.time}
			subject={message.subject}
			starred={message.starred}
			attachments={message.attachments}
			invitations={shown.invitations}
			open={{
				expanded: shown.expanded,
				preview: message.preview,
				toggle: () => send(shown.toggle),
			}}
		>
			{shown.body}
		</MessageCard>
	);
}

function ReplyAside({
	reply,
	threadId,
}: {
	reply: ReplyPanel | undefined;
	threadId: string;
}) {
	return (
		<aside className="panel thread-reply" aria-label="Reply with AI">
			<ReplyState reply={reply} form="aside" threadId={threadId} />
		</aside>
	);
}

// The panel's head: its title and, once AI can write, the model menu.
function ReplyHead({
	title,
	form,
	ready,
}: {
	title: string;
	form: "aside" | "sheet";
	ready?: ComposeReady;
}) {
	return (
		<div className={cx("reply-head", form === "sheet" && "reply-head-sheet")}>
			<span className="h2 grow trunc reply-title">{title}</span>
			{ready && <ReplyModelMenu ready={ready} />}
		</div>
	);
}

// The panel's state, in the desktop aside or the phone sheet.
function ReplyState({
	reply,
	form,
	threadId,
}: {
	reply: ReplyPanel | undefined;
	form: "aside" | "sheet";
	threadId: string;
}) {
	const state = reply?.state;
	switch (state?.case) {
		case "off":
			return (
				<>
					{form === "aside" && <ReplyHead title="Reply" form="aside" />}
					<ComposeOff />
				</>
			);
		case "ready":
			return form === "aside" ? (
				<>
					<ReplyHead title="Reply" form="aside" ready={state.value} />
					<ReadyAside ready={state.value} threadId={threadId} />
				</>
			) : (
				<PhoneReady ready={state.value} threadId={threadId} />
			);
		case undefined:
			return unset("ReplyPanel.state");
	}
}

function ComposeOff() {
	const send = useSend();
	return (
		<div className="compose-off settle">
			<span className="strong">AI compose is off</span>
			<span className="sm muted">
				Add your own {anyOf(modelProviders)} key to write replies with AI. Your
				system agents keep running their built-in checks
			</span>
			<Button variant="soft" onClick={() => send(modelSettings)}>
				Add a key
			</Button>
		</div>
	);
}

function ReplyModelMenu({ ready }: { ready: ComposeReady }) {
	const send = useSend();
	return (
		<ModelMenu
			label="Model"
			align="end"
			className="model-menu"
			rows={choiceRows(ready.models)}
			onPick={(modelName) =>
				send({ case: "selectComposeModel", value: { modelName } })
			}
			trigger={(trigger) => (
				<button type="button" className="chip press model-chip" {...trigger}>
					<ModelChip model={ready.model} />
					<Icon name="chevron-down" size="sm" className="subtle" />
				</button>
			)}
		/>
	);
}

function ReadyAside({
	ready,
	threadId,
}: {
	ready: ComposeReady;
	threadId: string;
}) {
	return (
		<>
			<div className="scroll reply-scroll">
				<div className="reply-group">
					<span className="label">Quick replies</span>
					<QuickReplies ready={ready} threadId={threadId} />
				</div>
				<div className="reply-group">
					<span className="label">Tone</span>
					<Tones ready={ready} />
				</div>
				<Overlap
					id={ready.generation.case ?? unset("ComposeReady.generation")}
					className="reply-generation"
				>
					<Generation ready={ready} threadId={threadId} />
				</Overlap>
			</div>
			<div className="reply-compose">
				<Instruction key={ready.revision} threadId={threadId} />
			</div>
		</>
	);
}

// A quick reply is written with its text as the instruction.
function QuickReplies({
	ready,
	threadId,
	touch,
}: {
	ready: ComposeReady;
	threadId: string;
	touch?: boolean;
}) {
	const send = useSend();
	return ready.quickReplies.map((text, i) => (
		<button
			key={text}
			type="button"
			className={cx("menu-row press row-in quick-reply", touch && "touch")}
			style={stagger(i)}
			onClick={() =>
				send({ case: "writeReply", value: { threadId, instruction: text } })
			}
		>
			{text}
		</button>
	));
}

// What the reply should say: Write (or ⌘Enter) writes it. The engine empties the box by a new
// revision once it took the instruction (ComposeReady.revision).
function Instruction({ threadId }: { threadId: string }) {
	const send = useSend();
	return (
		<MessageBox
			label="What should the reply say"
			placeholder="What should it say?"
			action="Write"
			onSubmit={(instruction) =>
				send({ case: "writeReply", value: { threadId, instruction } })
			}
		/>
	);
}

function Tones({ ready, touch }: { ready: ComposeReady; touch?: boolean }) {
	const send = useSend();
	return (
		<Segmented
			kind="radio"
			label="Tone"
			touch={touch}
			segments={ready.tones.map((tone) => ({
				key: tone.label,
				label: tone.label,
				selected: tone.selected,
				onSelect: () =>
					send({ case: "selectTone", value: { tone: tone.label } }),
			}))}
		/>
	);
}

function Generation({
	ready,
	threadId,
}: {
	ready: ComposeReady;
	threadId: string;
}) {
	const send = useSend();
	const { generation } = ready;
	const clear = (label: string) => (
		<Button
			variant="ghost"
			onClick={() => send({ case: "clearReply", value: { threadId } })}
		>
			{label}
		</Button>
	);
	switch (generation.case) {
		case "empty":
			return (
				<div className="tile generation generation-empty sm">
					{generation.value.hint}
				</div>
			);
		case "writing":
			return (
				<div className="tile generation generation-line">
					<span className="shimmer sm grow">{generation.value.label}</span>
					{clear("Stop")}
				</div>
			);
		case "failed":
			return (
				<div className="tile generation generation-line">
					<span className="sm bad grow settle" role="alert">
						{generation.value.reason}
					</span>
					{clear("Clear")}
				</div>
			);
		case "draft":
			return (
				<div className="tile generation">
					<span className="xs subtle">{generation.value.addressing}</span>
					<Paragraphs texts={generation.value.paragraphs} settle />
					<span className="settle">
						<Button
							variant="primary"
							onClick={() => send({ case: "sendReply", value: { threadId } })}
						>
							<Icon name="send" />
							Send
						</Button>
						<Button
							variant="soft"
							onClick={() => send({ case: "useReply", value: { threadId } })}
						>
							Edit in composer
						</Button>
						<span className="grow" />
						{clear("Clear")}
					</span>
				</div>
			);
		case undefined:
			return unset("ComposeReady.generation");
	}
}

function PhoneThread({ view }: { view: ThreadView }) {
	const send = useSend();
	const sheet = useOverlay();
	return (
		<div className="thread-phone">
			<PhoneHead
				bar={
					<MailToolbar
						threadId={view.threadId}
						actions={view.actions}
						place={view.place}
						star={{ messageId: view.messageId, starred: view.starred }}
						lead={
							<IconButton
								icon="chevron-left"
								label="Back"
								size="lg"
								touch
								onClick={() => send(back)}
							/>
						}
					/>
				}
			/>
			<div className="scroll noscroll thread-phone-scroll">
				<ThreadContent view={view} />
			</div>
			<div className="thread-phone-foot">
				<Button
					variant="soft"
					size="lg"
					className="reply-open"
					aria-haspopup="dialog"
					onClick={() => sheet.dispatch("toggle")}
				>
					<Icon name="compose" className="accent" />
					Reply with AI
				</Button>
			</div>
			<UserSheet overlay={sheet} label="Reply with AI">
				<ReplyState reply={view.reply} form="sheet" threadId={view.threadId} />
			</UserSheet>
		</div>
	);
}

function PhoneReady({
	ready,
	threadId,
}: {
	ready: ComposeReady;
	threadId: string;
}) {
	const { generation } = ready;
	return (
		<div className="phone-reply">
			<ReplyHead title={ready.title} form="sheet" ready={ready} />
			<Overlap id={generation.case ?? unset("ComposeReady.generation")}>
				{match(generation, "ComposeReady.generation", {
					empty: () => (
						<div className="reply-group">
							<QuickReplies ready={ready} threadId={threadId} touch />
						</div>
					),
					writing: () => <Generation ready={ready} threadId={threadId} />,
					failed: () => <Generation ready={ready} threadId={threadId} />,
					draft: () => <Generation ready={ready} threadId={threadId} />,
				})}
			</Overlap>
			<Tones ready={ready} touch />
			<Instruction key={ready.revision} threadId={threadId} />
		</div>
	);
}
