import "./inbox.css";
import { MailboxMenu } from "../chrome/MailboxMenu";
import { unset, useSend } from "../engine";
import {
	type InboxView,
	type MailRow,
	NoteKind,
	type ReadingPreview,
	type RowNote,
} from "../gen/ui/v1/inbox_pb";
import { useLayout } from "../presentation";
import { AgentDraft } from "../ui/AgentDraft";
import { Button, IconButton } from "../ui/Button";
import { cx } from "../ui/cx";
import { Dot } from "../ui/Dot";
import { Icon } from "../ui/Icon";
import { Logo } from "../ui/Logo";
import { ListPane, MailHeader, MailListRow, ReadingPane } from "../ui/MailList";
import { MessageBody } from "../ui/MessageBody";
import { Row, Rows } from "../ui/Rows";
import { Segmented } from "../ui/Segmented";
import { useSelectionGroup } from "../ui/Selection";
import { SelectList, useOptionIds } from "../ui/SelectList";
import { Summary } from "../ui/Summary";
import type { ScreenProps } from "./Screen";

// A mailbox or folder. Split layout: the list and the reading pane, where a row click selects.
// Without the reading pane (narrow desktop, phone) a row click opens the thread.
// Archive, snooze, star, search and compose have no intent yet and are drawn without one.
type Opens = "select" | "open";

export function Inbox({ view, nav }: ScreenProps<InboxView>) {
	const layout = useLayout();
	switch (layout) {
		case "phone":
			return <PhoneInbox view={view} nav={nav} />;
		case "list":
			return <ListPanel view={view} opens="open" />;
		case "split":
			return (
				<>
					<ListPanel view={view} opens="select" />
					<Read reading={view.reading} />
				</>
			);
	}
}

function Categories({
	categories,
	touch,
}: {
	categories: InboxView["categories"];
	touch?: boolean;
}) {
	const send = useSend();
	switch (categories.case) {
		case "tabs":
			return (
				<Segmented
					kind="tabs"
					label="Inbox view"
					touch={touch}
					segments={categories.value.tabs.map((tab) => ({
						key: String(tab.category),
						label: tab.label,
						selected: tab.selected,
						onSelect: () =>
							send({
								case: "selectCategory",
								value: { category: tab.category },
							}),
					}))}
				/>
			);
		case "uncategorized":
			return null;
		case undefined:
			return unset("InboxView.categories");
	}
}

// Beside the reading pane the list is a listbox: arrows select, Enter opens the selected thread.
// Without it a row opens its thread, so the list is a plain list.
function ListPanel({ view, opens }: { view: InboxView; opens: Opens }) {
	const send = useSend();
	const group = useSelectionGroup();
	const optionId = useOptionIds();
	const selected = view.rows.find((row) => row.selected);
	const rows = (listbox: boolean) => (
		<Rows label="Messages" group={listbox}>
			{view.rows.map((row, i) => (
				<Row
					key={row.threadId}
					index={i}
					option={
						listbox
							? { id: optionId(row.threadId), selected: row.selected }
							: undefined
					}
				>
					<DesktopRow row={row} opens={opens} group={group} />
				</Row>
			))}
		</Rows>
	);
	return (
		<ListPane
			label={view.title}
			head={
				<>
					<div className="list-title">
						<h1 className="h2 grow trunc">{view.title}</h1>
						<IconButton icon="search" label="Search" />
					</div>
					<Categories categories={view.categories} />
				</>
			}
		>
			{opens === "select" ? (
				<SelectList
					label={view.title}
					active={selected && optionId(selected.threadId)}
					onOpen={
						selected &&
						(() =>
							send({
								case: "openThread",
								value: { threadId: selected.threadId },
							}))
					}
				>
					{rows(true)}
				</SelectList>
			) : (
				rows(false)
			)}
		</ListPane>
	);
}

function useRowIntent(opens: Opens) {
	const send = useSend();
	return (threadId: string) => () =>
		send(
			opens === "select"
				? { case: "selectThread", value: { threadId } }
				: { case: "openThread", value: { threadId } },
		);
}

function DesktopRow({
	row,
	opens,
	group,
}: {
	row: MailRow;
	opens: Opens;
	group: string;
}) {
	const intent = useRowIntent(opens);
	return (
		<MailListRow
			group={group}
			selected={row.selected}
			tone={row.unread ? "unread" : "idle"}
			from={row.from}
			unread={row.unread}
			provider={row.provider}
			account={row.account}
			time={row.time}
			onSelect={intent(row.threadId)}
			actions={
				<>
					<IconButton icon="archive" label="Archive" />
					<IconButton icon="snooze" label="Snooze" aria-haspopup="menu" />
				</>
			}
		>
			<span className="trunc sm">{row.subject}</span>
			<Note note={row.note} />
		</MailListRow>
	);
}

function Note({ note }: { note: RowNote | undefined }) {
	if (!note) return null;
	return (
		<span className={cx("trunc xs", noteClass(note.kind))}>{note.text}</span>
	);
}

function noteClass(kind: NoteKind) {
	switch (kind) {
		case NoteKind.WAITING:
			return "note-waiting";
		case NoteKind.FLAGGED:
			return "accent";
		case NoteKind.FILED:
			return "subtle";
		case NoteKind.UNSPECIFIED:
			return unset("NoteKind");
	}
}

function Read({ reading }: { reading: InboxView["reading"] }) {
	const send = useSend();
	const id = reading.case === "preview" ? reading.value.threadId : reading.case;
	return (
		<ReadingPane
			id={id ?? unset("InboxView.reading")}
			bar={
				<>
					<IconButton icon="archive" label="Archive" />
					<IconButton icon="star" label="Star" />
					{reading.case === "preview" && (
						<Button
							variant="ghost"
							onClick={() =>
								send({
									case: "openThread",
									value: { threadId: reading.value.threadId },
								})
							}
						>
							Open thread
						</Button>
					)}
				</>
			}
		>
			<Reading reading={reading} />
		</ReadingPane>
	);
}

function Reading({ reading }: { reading: InboxView["reading"] }) {
	switch (reading.case) {
		case "preview":
			return <Preview preview={reading.value} />;
		case "nothing":
			return null;
		case undefined:
			return unset("InboxView.reading");
	}
}

function Preview({ preview }: { preview: ReadingPreview }) {
	const send = useSend();
	return (
		<article className="read-article">
			<MailHeader
				subject={preview.subject}
				from={preview.from}
				toLine={preview.toLine}
				time={preview.time}
			/>
			<Summary summary={preview.summary} label="Summary by Scribe" />
			<MessageBody url={preview.bodyUrl} title={preview.subject} />
			{preview.drafts.map((draft) => (
				<AgentDraft
					key={draft.approvalId}
					draft={draft}
					onEditWithAi={() =>
						send({
							case: "openThread",
							value: { threadId: preview.threadId },
						})
					}
				/>
			))}
		</article>
	);
}

function PhoneInbox({ view, nav }: ScreenProps<InboxView>) {
	return (
		<div className="inbox-phone">
			<header className="phone-head">
				<div className="phone-bar">
					<Logo brand={{ kind: "mark" }} size={24} />
					<span className="grow">
						<MailboxMenu nav={nav} title={view.title} />
					</span>
					<IconButton icon="search" label="Search" size="lg" touch />
				</div>
				<Categories categories={view.categories} touch />
			</header>
			<div className="scroll noscroll inbox-phone-scroll">
				<Rows label="Messages" className="inbox-phone-rows">
					{view.rows.map((row, i) => (
						<Row key={row.threadId} index={i}>
							<PhoneRow row={row} />
						</Row>
					))}
				</Rows>
			</div>
			<button
				type="button"
				className="btn primary press fab"
				aria-label="Compose"
			>
				<Icon name="compose" size="lg" />
			</button>
		</div>
	);
}

function PhoneRow({ row }: { row: MailRow }) {
	const intent = useRowIntent("open");
	return (
		<button
			type="button"
			className="mrow phone-row"
			onClick={intent(row.threadId)}
		>
			<Dot tone={row.unread ? "unread" : "idle"} className="row-dot" />
			<span className="row-main">
				<span className={cx("trunc phone-from", row.unread && "unread")}>
					{row.from}
				</span>
				<span className="trunc phone-subject">{row.subject}</span>
				<Note note={row.note} />
			</span>
			<span className="xs subtle row-time">{row.time}</span>
		</button>
	);
}
