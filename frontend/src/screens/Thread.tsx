import "./thread.css";
import { modelSettings } from "../chrome/places";
import { unset, useSend } from "../engine";
import type {
	ComposeReady,
	ModelChoice,
	ReplyPanel,
	ThreadMessage,
	ThreadView,
} from "../gen/ui/v1/thread_pb";
import { useLayout, useOverlay } from "../presentation";
import { AgentDraft } from "../ui/AgentDraft";
import { Button, IconButton } from "../ui/Button";
import { Check } from "../ui/Check";
import { Composer } from "../ui/Composer";
import { Icon } from "../ui/Icon";
import { Invitation } from "../ui/Invitation";
import { Logo } from "../ui/Logo";
import { MessageBody } from "../ui/MessageBody";
import { Overlap } from "../ui/Overlap";
import { Popover, UserSheet } from "../ui/Overlay";
import { Paragraphs } from "../ui/Paragraphs";
import { Segmented } from "../ui/Segmented";
import { Summary } from "../ui/Summary";
import { stagger } from "../ui/stagger";
import type { ScreenProps } from "./Screen";

// One thread with Reply with AI beside it (desktop) or in a sheet (phone). Back returns to the
// inbox. Archive, more, forward, quick replies, tone, the model choice, Write, Attach and Send have
// no intent yet and are drawn without one; the reply instruction stays in the field.
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
					<ReplyAside reply={view.reply} />
				</>
			);
	}
}

const back = { case: "back", value: {} } as const;

function ThreadPanel({ view }: { view: ThreadView }) {
	const send = useSend();
	return (
		<section className="panel thread-main" aria-label={view.subject}>
			<div className="thread-bar">
				<IconButton
					icon="chevron-left"
					label="Back to inbox"
					onClick={() => send(back)}
				/>
				<span className="sm subtle grow trunc">{view.context}</span>
				<IconButton icon="archive" label="Archive" />
				<IconButton icon="more" label="More" />
			</div>
			<div className="scroll thread-scroll">
				<ThreadContent view={view} />
			</div>
		</section>
	);
}

function ThreadContent({ view }: { view: ThreadView }) {
	return (
		<div className="thread-body">
			<h1 className="h1 thread-subject">{view.subject}</h1>
			<Summary summary={view.summary} label="Thread summary by Scribe" />
			{view.messages.map((message) => (
				<Message key={message.messageId} message={message} />
			))}
			{view.drafts.map((draft) => (
				<AgentDraft key={draft.approvalId} draft={draft} />
			))}
		</div>
	);
}

function Message({ message }: { message: ThreadMessage }) {
	const send = useSend();
	const { display } = message;
	const messageId = message.messageId;
	return (
		<div className="message">
			<button
				type="button"
				className="press message-head"
				aria-expanded={display.case === "expanded"}
				onClick={() =>
					send(
						display.case === "expanded"
							? { case: "collapseMessage", value: { messageId } }
							: { case: "expandMessage", value: { messageId } },
					)
				}
			>
				<span className="grow two-line message-who">
					<span className="sender">{message.from}</span>
					<span className="xs subtle trunc">{message.preview}</span>
				</span>
				<span className="xs subtle">{message.time}</span>
			</button>
			<MessageDisplay display={display} title={message.preview} />
		</div>
	);
}

function MessageDisplay({
	display,
	title,
}: {
	display: ThreadMessage["display"];
	title: string;
}) {
	switch (display.case) {
		case "collapsed":
			return null;
		case "expanded":
			return (
				<div className="message-open reveal">
					<MessageBody url={display.value.bodyUrl} title={title} />
					{display.value.invitations.map((invitation) => (
						<Invitation
							key={invitation.eventId}
							invitation={invitation}
							form="thread"
						/>
					))}
				</div>
			);
		case undefined:
			return unset("ThreadMessage.display");
	}
}

function ReplyAside({ reply }: { reply: ReplyPanel | undefined }) {
	return (
		<aside className="panel thread-reply" aria-label="Reply with AI">
			<ReplyState reply={reply} form="aside" />
		</aside>
	);
}

// The panel's state, in the desktop aside or the phone sheet.
function ReplyState({
	reply,
	form,
}: {
	reply: ReplyPanel | undefined;
	form: "aside" | "sheet";
}) {
	const state = reply?.state;
	switch (state?.case) {
		case "off":
			return (
				<>
					{form === "aside" && (
						<div className="reply-head">
							<span className="h2 grow reply-title">Reply</span>
						</div>
					)}
					<ComposeOff />
				</>
			);
		case "ready":
			return form === "aside" ? (
				<>
					<div className="reply-head">
						<span className="h2 grow reply-title">Reply</span>
						<ModelMenu ready={state.value} />
					</div>
					<ReadyAside ready={state.value} />
				</>
			) : (
				<PhoneReady ready={state.value} />
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
				Add your own Anthropic, OpenAI, Gemini or OpenRouter key to write
				replies with AI. Your system agents keep running their built-in checks
			</span>
			<Button variant="soft" onClick={() => send(modelSettings)}>
				Add a key
			</Button>
		</div>
	);
}

function ModelMenu({ ready }: { ready: ComposeReady }) {
	return (
		<Popover
			label="Model"
			align="end"
			className="model-menu"
			trigger={(trigger) => (
				<button type="button" className="chip press model-chip" {...trigger}>
					<ModelLogo model={ready.model} />
					{ready.model?.name}
					<Icon name="chevron-down" size="sm" className="subtle" />
				</button>
			)}
		>
			{(dismiss) =>
				ready.models.map((choice, i) => (
					<ModelOption
						key={choice.model?.name}
						choice={choice}
						index={i}
						dismiss={dismiss}
					/>
				))
			}
		</Popover>
	);
}

function ModelLogo({ model }: { model: ComposeReady["model"] }) {
	if (!model) return unset("ComposeReady.model");
	return <Logo brand={{ kind: "model", provider: model.provider }} />;
}

function ModelOption({
	choice,
	index,
	dismiss,
}: {
	choice: ModelChoice;
	index: number;
	dismiss: () => void;
}) {
	const send = useSend();
	const model = choice.model ?? unset("ModelChoice.model");
	switch (choice.availability.case) {
		case "ready":
			return (
				<button
					type="button"
					role="menuitemradio"
					aria-checked={choice.selected}
					className="menu-row row-in stagger"
					style={stagger(index)}
				>
					<ModelLogo model={model} />
					<span className="grow">{model.name}</span>
					<span className="xs subtle">
						{choice.availability.value.providerName}
					</span>
					<Check on={choice.selected} />
				</button>
			);
		case "needsKey":
			return (
				<button
					type="button"
					role="menuitem"
					className="menu-row row-in stagger subtle"
					style={stagger(index)}
					onClick={() => {
						dismiss();
						send(modelSettings);
					}}
				>
					<ModelLogo model={model} />
					<span className="grow">{model.name}</span>
					<span className="xs">{choice.availability.value.label}</span>
				</button>
			);
		case undefined:
			return unset("ModelChoice.availability");
	}
}

function ReadyAside({ ready }: { ready: ComposeReady }) {
	return (
		<>
			<div className="scroll reply-scroll">
				<div className="reply-group">
					<span className="label">Quick replies</span>
					{ready.quickReplies.map((text, i) => (
						<button
							key={text}
							type="button"
							className="menu-row press row-in stagger quick-reply"
							style={stagger(i)}
						>
							{text}
						</button>
					))}
				</div>
				<div className="reply-group">
					<span className="label">Tone</span>
					<Tones ready={ready} />
				</div>
				<Overlap
					id={ready.generation.case ?? unset("ComposeReady.generation")}
					className="reply-generation"
				>
					<Generation ready={ready} />
				</Overlap>
			</div>
			<div className="reply-compose">
				<Composer
					label="What should the reply say"
					placeholder="What should it say?"
				>
					<IconButton icon="plus" label="Attach" />
					<span className="grow" />
					<Button variant="ghost">Write</Button>
					<Button variant="primary">Send</Button>
				</Composer>
			</div>
		</>
	);
}

function Tones({ ready, touch }: { ready: ComposeReady; touch?: boolean }) {
	return (
		<Segmented
			kind="radio"
			label="Tone"
			touch={touch}
			segments={ready.tones.map((tone) => ({
				key: tone.label,
				label: tone.label,
				selected: tone.selected,
			}))}
		/>
	);
}

function Generation({ ready }: { ready: ComposeReady }) {
	const { generation } = ready;
	switch (generation.case) {
		case "empty":
			return (
				<div className="tile generation generation-empty sm">
					{generation.value.hint}
				</div>
			);
		case "writing":
			return (
				<div className="tile generation">
					<span className="shimmer sm">{generation.value.label}</span>
				</div>
			);
		case "draft":
			return (
				<div className="tile generation">
					<span className="xs subtle">{generation.value.addressing}</span>
					<Paragraphs texts={generation.value.paragraphs} settle />
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
			<header className="thread-phone-bar">
				<IconButton
					icon="chevron-left"
					label="Back"
					size="lg"
					touch
					onClick={() => send(back)}
				/>
				<span className="grow" />
				<IconButton icon="archive" label="Archive" size="lg" touch />
				<IconButton icon="more" label="More" size="lg" touch />
			</header>
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
				<IconButton
					icon="forward"
					label="Forward"
					size="lg"
					touch
					className="raised"
				/>
			</div>
			<UserSheet overlay={sheet} label="Reply with AI">
				<ReplyState reply={view.reply} form="sheet" />
			</UserSheet>
		</div>
	);
}

function PhoneReady({ ready }: { ready: ComposeReady }) {
	const { generation } = ready;
	return (
		<div className="phone-reply">
			<div className="reply-head-phone">
				<span className="h2 grow trunc">{ready.title}</span>
				<span className="chip model-chip">
					<ModelLogo model={ready.model} />
					{ready.model?.name}
				</span>
			</div>
			<Overlap id={generation.case ?? unset("ComposeReady.generation")}>
				{generation.case === "empty" ? (
					<div className="reply-group">
						{ready.quickReplies.map((text, i) => (
							<button
								key={text}
								type="button"
								className="menu-row touch press row-in stagger quick-reply"
								style={stagger(i)}
							>
								{text}
							</button>
						))}
					</div>
				) : (
					<Generation ready={ready} />
				)}
			</Overlap>
			<Tones ready={ready} touch />
			<Button variant="primary" size="lg">
				Send
			</Button>
		</div>
	);
}
