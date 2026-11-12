import type { MessageInitShape } from "@bufbuild/protobuf";
import type { ReactNode } from "react";
import { unset, useSend } from "../engine";
import {
	type Attachment,
	AttachmentKind,
	type Invitation as InvitationView,
	type MailActions,
	type MoveChoice,
	type SnoozeChoice,
} from "../gen/ui/v1/common_pb";
import type { OpenComposerSchema } from "../gen/ui/v1/shell_pb";
import { IconButton } from "./Button";
import { cx } from "./cx";
import { Icon, type IconName } from "./Icon";
import { Invitation } from "./Invitation";
import { Popover } from "./Overlay";
import { stagger } from "./stagger";

// A mail client's thread toolbar and message card, shared by the thread and the inbox reading
// pane. The card's rounded border and its hairlines are there because the owner asked for each
// message to read as one card: sender line, subject, body, attachments and invitations.

// Archive, delete, snooze and move (each a menu from its trigger), mark unread or read, then
// where the thread is. Archive and delete show where the engine offers them (MailActions).
export function MailToolbar({
	threadId,
	actions,
	place,
	star,
	lead,
	trail,
}: {
	threadId: string;
	actions: MailActions | undefined;
	place: string;
	// The thread's star, on its latest message.
	star?: { messageId: string; starred: boolean };
	lead?: ReactNode;
	trail?: ReactNode;
}) {
	const send = useSend();
	const mail = actions ?? unset("MailActions");
	return (
		<div className="mail-toolbar" role="toolbar" aria-label="Thread">
			{lead}
			{mail.archive && (
				<IconButton
					icon="archive"
					label="Archive"
					onClick={() => send({ case: "archiveThread", value: { threadId } })}
				/>
			)}
			{mail.trash && (
				<IconButton
					icon="trash"
					label="Delete"
					onClick={() => send({ case: "trashThread", value: { threadId } })}
				/>
			)}
			<SnoozeMenu threadId={threadId} choices={mail.snooze} />
			<MoveMenu threadId={threadId} choices={mail.move} />
			<IconButton
				icon="unread"
				label={mail.unread ? "Mark read" : "Mark unread"}
				aria-pressed={mail.unread}
				onClick={() =>
					send({ case: "setUnread", value: { threadId, unread: !mail.unread } })
				}
			/>
			{star && <Star messageId={star.messageId} starred={star.starred} />}
			<span className="xs subtle grow trunc mail-place">{place}</span>
			{trail}
		</div>
	);
}

// The snooze presets, from the trigger; none in the trash.
export function SnoozeMenu({
	threadId,
	choices,
	align = "start",
	tabIndex,
}: {
	threadId: string;
	choices: SnoozeChoice[];
	align?: "start" | "end";
	// -1 in a list row, whose actions stay out of the tab order (SelectList.tsx).
	tabIndex?: -1;
}) {
	const send = useSend();
	if (choices.length === 0) return null;
	return (
		<Popover
			label="Snooze until"
			align={align}
			className="mail-menu"
			trigger={(props) => (
				<IconButton
					{...props}
					icon="snooze"
					label="Snooze"
					tabIndex={tabIndex}
				/>
			)}
		>
			{(dismiss) =>
				choices.map((choice, i) => (
					<button
						key={choice.preset}
						type="button"
						role="menuitem"
						className="menu-row press row-in"
						style={stagger(i)}
						onClick={() => {
							dismiss();
							send({
								case: "snoozeThread",
								value: { threadId, preset: choice.preset },
							});
						}}
					>
						<span className="grow">{choice.label}</span>
						<span className="xs subtle">{choice.when}</span>
					</button>
				))
			}
		</Popover>
	);
}

function MoveMenu({
	threadId,
	choices,
}: {
	threadId: string;
	choices: MoveChoice[];
}) {
	const send = useSend();
	return (
		<Popover
			label="Move to"
			className="mail-menu"
			trigger={(props) => (
				<IconButton {...props} icon="folder" label="Move to folder" />
			)}
		>
			{(dismiss) =>
				choices.map((choice, i) => (
					<button
						key={choice.to.case === "folderId" ? choice.to.value : "inbox"}
						type="button"
						role="menuitemradio"
						aria-checked={choice.current}
						className="menu-row press row-in"
						style={stagger(i)}
						onClick={() => {
							dismiss();
							send({ case: "moveThread", value: { threadId, to: choice.to } });
						}}
					>
						<span className="grow">{choice.label}</span>
						{choice.current && <Icon name="check" size="sm" />}
					</button>
				))
			}
		</Popover>
	);
}

// The outline and the filled star overlap and crossfade with a blur (`.swap`) as it toggles.
export function Star({
	messageId,
	starred,
	tabIndex,
}: {
	messageId: string;
	starred: boolean;
	tabIndex?: -1;
}) {
	const send = useSend();
	return (
		<button
			type="button"
			className={cx("icon-btn press", starred && "starred")}
			aria-label={starred ? "Starred" : "Star"}
			aria-pressed={starred}
			tabIndex={tabIndex}
			onClick={() =>
				send({ case: "setStarred", value: { messageId, starred: !starred } })
			}
		>
			<span className="swap">
				<Icon name="star" size="sm" className={cx(starred && "hide")} />
				<Icon
					name="star"
					size="sm"
					className={cx("filled", !starred && "hide")}
				/>
			</span>
		</button>
	);
}

// The card's head: who sent it to whom and when, with the message's actions. `open` is the head's
// own control (expand or collapse in a thread); without it the head is plain text.
export function MessageCard({
	messageId,
	from,
	recipients,
	time,
	subject,
	starred,
	attachments,
	invitations,
	open,
	children,
}: {
	messageId: string;
	from: string;
	recipients: string;
	time: string;
	subject: string;
	starred: boolean;
	attachments: Attachment[];
	invitations: InvitationView[];
	open?: { expanded: boolean; preview: string; toggle: () => void };
	children?: ReactNode;
}) {
	const who = (
		<span className="two-line grow card-who">
			<span className="sender trunc">{from}</span>
			<span className="xs subtle trunc">
				{open && !open.expanded ? open.preview : recipients}
			</span>
		</span>
	);
	const expanded = open ? open.expanded : true;
	const send = useSend();
	const compose = (kind: OpenComposerKind) =>
		send({ case: "openComposer", value: { kind } });
	return (
		<article className={cx("mail-card", !expanded && "collapsed")}>
			<header className="card-head">
				{open ? (
					<button
						type="button"
						className="press card-toggle"
						aria-expanded={open.expanded}
						onClick={open.toggle}
					>
						{who}
					</button>
				) : (
					who
				)}
				<span className="xs subtle card-time">{time}</span>
				{expanded && (
					<span className="card-actions">
						<IconButton
							icon="reply"
							label="Reply"
							size="sm"
							onClick={() =>
								compose({ case: "replyToMessageId", value: messageId })
							}
						/>
						<IconButton
							icon="reply-all"
							label="Reply all"
							size="sm"
							onClick={() =>
								compose({ case: "replyAllToMessageId", value: messageId })
							}
						/>
						<IconButton
							icon="forward"
							label="Forward"
							size="sm"
							onClick={() =>
								compose({ case: "forwardMessageId", value: messageId })
							}
						/>
						<Star messageId={messageId} starred={starred} />
					</span>
				)}
			</header>
			{expanded && (
				<div className="reveal card-open">
					<h3 className="card-subject">{subject}</h3>
					<div className="card-body">{children}</div>
					{attachments.length > 0 && (
						<ul className="attachments" aria-label="Attachments">
							{attachments.map((attachment) => (
								<li key={attachment.id} className="attachment">
									<Icon name={attachmentIcon(attachment.kind)} size="lg" />
									<span className="two-line">
										<span className="sm trunc">{attachment.name}</span>
										<span className="xs subtle">{attachment.size}</span>
									</span>
								</li>
							))}
						</ul>
					)}
					{invitations.map((invitation) => (
						<Invitation
							key={invitation.eventId}
							invitation={invitation}
							form="thread"
						/>
					))}
				</div>
			)}
		</article>
	);
}

type OpenComposerKind = NonNullable<
	MessageInitShape<typeof OpenComposerSchema>["kind"]
>;

function attachmentIcon(kind: AttachmentKind): IconName {
	switch (kind) {
		case AttachmentKind.IMAGE:
			return "image";
		case AttachmentKind.PDF:
		case AttachmentKind.DOCUMENT:
			return "file-text";
		case AttachmentKind.SPREADSHEET:
			return "table";
		case AttachmentKind.PRESENTATION:
			return "presentation";
		case AttachmentKind.ARCHIVE:
			return "file-zip";
		case AttachmentKind.CALENDAR:
			return "calendar";
		case AttachmentKind.OTHER:
			return "file";
		case AttachmentKind.UNSPECIFIED:
			return unset("AttachmentKind");
	}
}
