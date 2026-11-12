import { AnimatePresence, m } from "motion/react";
import { useLayoutEffect, useRef } from "react";
import { unset, useSend } from "../engine";
import type { Composer, ComposerOpen } from "../gen/ui/v1/compose_pb";
import { takePress, useDebounce, useDraft, useLayout } from "../presentation";
import { Button, IconButton } from "../ui/Button";
import { Confirm } from "../ui/Confirm";
import { cx } from "../ui/cx";
import { Icon } from "../ui/Icon";
import { useTiming } from "../ui/motion";
import { OutcomeNote, pulseOn, useOutcome } from "../ui/Outcome";
import { Popover } from "../ui/Overlay";

// The one composer, over any screen: docked at the bottom right on desktop, full screen on the
// phone. The engine opens it (ShellView.composer); the typed text is input sent with update_draft
// on a debounce, and flushed before Send, Send later and Close so the engine sends what was typed.
// Attachments come from the platform file picker; only a file's name and size are passed on.
export function ComposeWindow({
	composer,
}: {
	composer: Composer | undefined;
}) {
	const state = composer?.state ?? unset("ShellView.composer");
	return <AnimatePresence>{open(state)}</AnimatePresence>;
}

function open(state: Composer["state"]) {
	switch (state.case) {
		case "closed":
			return null;
		case "open":
			return <Window key={state.value.draftId} draft={state.value} />;
		case undefined:
			return unset("Composer.state");
	}
}

// The composer grows from the control pressed to open it and shrinks back there when it closes
// (motion rules 11 and 12); opened by a key it grows from its docked corner (`.compose-window`'s
// transform origin). The origin is placed once, on the element, before the first paint.
function Window({ draft }: { draft: ComposerOpen }) {
	const { enter, exit } = useTiming();
	const phone = useLayout() === "phone";
	const panel = useRef<HTMLElement>(null);
	useLayoutEffect(() => {
		const element = panel.current;
		const press = takePress();
		if (!element) return;
		switch (press.kind) {
			case "none":
				return;
			case "at": {
				// the untransformed box: Motion's initial scale is already on the element
				const parent = (
					element.offsetParent ?? document.body
				).getBoundingClientRect();
				const left = parent.left + element.offsetLeft;
				const top = parent.top + element.offsetTop;
				element.style.transformOrigin = `${press.x - left}px ${press.y - top}px`;
				return;
			}
		}
	}, []);
	return (
		<m.section
			ref={panel}
			className={cx("compose-window", phone && "compose-phone")}
			aria-label={draft.title}
			initial={{ opacity: 0, scale: 0.6 }}
			animate={{ opacity: 1, scale: 1 }}
			exit={{ opacity: 0, scale: 0.6, transition: exit }}
			transition={enter}
		>
			<Form draft={draft} />
		</m.section>
	);
}

function Form({ draft }: { draft: ComposerOpen }) {
	const send = useSend();
	const { draftId } = draft;
	const [to, setTo] = useDraft(draft.to);
	const [subject, setSubject] = useDraft(draft.subject);
	const [body, setBody] = useDraft(draft.body);
	const { schedule, flush } = useDebounce();
	const picker = useRef<HTMLInputElement>(null);
	const outcome = useOutcome({ case: "draftId", value: draftId });
	const save = (next: { to: string; subject: string; body: string }) =>
		schedule(() => send({ case: "updateDraft", value: { draftId, ...next } }));
	const then = (run: () => void) => () => {
		flush();
		run();
	};
	return (
		<>
			<header className="compose-head">
				<h2 className="strong grow trunc">{draft.title}</h2>
				<span className="xs subtle">{draft.saved}</span>
				<IconButton
					icon="close"
					label="Close and keep the draft"
					onClick={then(() => send({ case: "closeComposer", value: {} }))}
				/>
			</header>
			<div className="compose-fields">
				<label className="compose-line">
					<span className="xs subtle compose-key">From</span>
					<select
						className="compose-from"
						value={
							(
								draft.from.find((each) => each.selected) ??
								unset("ComposerOpen.from selected")
							).id
						}
						onChange={(event) =>
							send({
								case: "setDraftAccount",
								value: { draftId, accountId: event.target.value },
							})
						}
					>
						{draft.from.map((each) => (
							<option key={each.id} value={each.id}>
								{each.label}
							</option>
						))}
					</select>
				</label>
				<label className="compose-line">
					<span className="xs subtle compose-key">To</span>
					<input
						// Opening takes focus: To for a new mail, the message for a reply.
						// biome-ignore lint/a11y/noAutofocus: the user opened the composer; focus moves into it
						autoFocus={draft.to === ""}
						value={to}
						onChange={(event) => {
							setTo(event.target.value);
							save({ to: event.target.value, subject, body });
						}}
					/>
				</label>
				<label className="compose-line">
					<span className="xs subtle compose-key">Subject</span>
					<input
						value={subject}
						onChange={(event) => {
							setSubject(event.target.value);
							save({ to, subject: event.target.value, body });
						}}
					/>
				</label>
				<textarea
					className="compose-body"
					aria-label="Message"
					// biome-ignore lint/a11y/noAutofocus: the user opened the composer; focus moves into it
					autoFocus={draft.to !== ""}
					value={body}
					onChange={(event) => {
						setBody(event.target.value);
						save({ to, subject, body: event.target.value });
					}}
				/>
				{draft.attachments.length > 0 && (
					<ul className="compose-files" aria-label="Attachments">
						{draft.attachments.map((file) => (
							<li key={file.id} className="chip compose-file settle">
								<Icon name="file" size="sm" />
								<span className="trunc">{file.name}</span>
								<span className="subtle">{file.size}</span>
								<IconButton
									icon="close"
									size="sm"
									label={`Remove ${file.name}`}
									onClick={() =>
										send({
											case: "removeAttachment",
											value: { draftId, attachmentId: file.id },
										})
									}
								/>
							</li>
						))}
					</ul>
				)}
			</div>
			<OutcomeNote outcome={outcome} className="compose-outcome" />
			<footer className="compose-foot">
				<Button
					variant="primary"
					className={pulseOn(outcome)}
					onClick={then(() => send({ case: "sendDraft", value: { draftId } }))}
				>
					<Icon name="send" />
					Send
				</Button>
				<Popover
					label="Send later"
					side="above"
					className="compose-later"
					trigger={(props) => (
						<IconButton {...props} icon="clock" label="Send later" />
					)}
				>
					{(dismiss) =>
						draft.sendLater.map((choice) => (
							<button
								key={choice.label}
								type="button"
								role="menuitem"
								className="menu-row press"
								onClick={then(() => {
									dismiss();
									send({
										case: "sendDraftLater",
										value: {
											draftId,
											dayId: choice.dayId,
											minute: choice.minute,
										},
									});
								})}
							>
								{choice.label}
							</button>
						))
					}
				</Popover>
				<IconButton
					icon="file"
					label="Attach files"
					onClick={() => picker.current?.click()}
				/>
				<input
					ref={picker}
					type="file"
					multiple
					hidden
					onChange={(event) => {
						for (const file of event.target.files ?? [])
							send({
								case: "attachFile",
								value: {
									draftId,
									name: file.name,
									sizeBytes: BigInt(file.size),
								},
							});
						event.target.value = "";
					}}
				/>
				<span className="grow" />
				<Confirm
					question="Discard this draft?"
					action="Discard"
					align="end"
					side="above"
					onConfirm={() => send({ case: "discardDraft", value: { draftId } })}
					trigger={(props) => (
						<IconButton {...props} icon="trash" label="Discard draft" />
					)}
				/>
			</footer>
		</>
	);
}
