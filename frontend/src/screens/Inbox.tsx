import "./inbox.css";
import type { ReactNode } from "react";
import { MailboxMenu } from "../chrome/MailboxMenu";
import { newMail } from "../chrome/places";
import { match, unset, useSend } from "../engine";
import { Form, type SnoozeChoice } from "../gen/ui/v1/common_pb";
import {
	type InboxView,
	type MailRow,
	NoteKind,
	type ReadingPreview,
} from "../gen/ui/v1/inbox_pb";
import { useDraft, useLayout } from "../presentation";
import { AgentDraft } from "../ui/AgentDraft";
import { Button, IconButton } from "../ui/Button";
import { cx } from "../ui/cx";
import { Dot } from "../ui/Dot";
import { Icon } from "../ui/Icon";
import { Logo } from "../ui/Logo";
import { MailToolbar, MessageCard, SnoozeMenu, Star } from "../ui/MailCard";
import { ListPane, MailListRow, ReadingPane } from "../ui/MailList";
import { MessageBody } from "../ui/MessageBody";
import { OutcomeNote, useOutcome } from "../ui/Outcome";
import { PhoneHead } from "../ui/PhoneHead";
import { Row, Rows } from "../ui/Rows";
import { Segmented } from "../ui/Segmented";
import { useSelectionGroup } from "../ui/Selection";
import { SelectList, tabStop } from "../ui/SelectList";
import { Summary } from "../ui/Summary";
import type { ScreenProps } from "./Screen";

// A mailbox or folder. Split layout: the list and the reading pane, where a row click selects.
// Without the reading pane (narrow desktop, phone) a row click opens the thread.
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

// The categories show unless a search is: the results are of every category.
function UnlessSearching({
	search,
	children,
}: {
	search: InboxView["search"];
	children: ReactNode;
}) {
	return match(search, "InboxView.search", {
		notSearching: () => children,
		results: () => null,
	});
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

// Beside the reading pane the list is a SelectList: one tab stop, arrows select, Enter opens the
// selected thread.
// Without it a row opens its thread, so the list is a plain list.
function ListPanel({ view, opens }: { view: InboxView; opens: Opens }) {
	const send = useSend();
	const group = useSelectionGroup();
	const selected = view.rows.find((row) => row.selected);
	const rows = (
		<Rows label="Messages">
			{view.rows.map((row, i) => (
				<Row key={row.threadId} index={i}>
					<DesktopRow
						row={row}
						opens={opens}
						group={group}
						snooze={view.snooze}
						tabStop={
							opens === "open" ||
							tabStop(row.selected, i, selected !== undefined)
						}
					/>
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
						<IconButton
							icon="compose"
							label="New mail"
							onClick={() => send(newMail)}
						/>
					</div>
					<Search search={view.search} />
					<UnlessSearching search={view.search}>
						<Categories categories={view.categories} />
					</UnlessSearching>
				</>
			}
		>
			{opens === "select" ? (
				<SelectList
					active={selected?.threadId}
					onOpen={
						selected &&
						(() =>
							send({
								case: "openThread",
								value: { threadId: selected.threadId },
							}))
					}
				>
					{rows}
				</SelectList>
			) : (
				rows
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
	tabStop,
	snooze,
}: {
	row: MailRow;
	opens: Opens;
	group: string;
	tabStop: boolean;
	snooze: SnoozeChoice[];
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
			tabStop={tabStop}
			actions={<RowActions row={row} snooze={snooze} />}
		>
			<span className="trunc sm">{row.subject}</span>
			<Note note={row.agentNote} />
		</MailListRow>
	);
}

// The row's hover actions: archive, delete, snooze where the row offers it, read or unread, and
// the latest message's star. The row leaves the list when the engine's next view no longer has
// it, and the rows after it close up.
function RowActions({ row, snooze }: { row: MailRow; snooze: SnoozeChoice[] }) {
	const send = useSend();
	const { threadId } = row;
	return (
		<>
			<IconButton
				icon="archive"
				label="Archive"
				tabIndex={-1}
				onClick={() => send({ case: "archiveThread", value: { threadId } })}
			/>
			<IconButton
				icon="trash"
				label="Delete"
				tabIndex={-1}
				onClick={() => send({ case: "trashThread", value: { threadId } })}
			/>
			<IconButton
				icon="unread"
				label={row.unread ? "Mark read" : "Mark unread"}
				tabIndex={-1}
				onClick={() =>
					send({ case: "setUnread", value: { threadId, unread: !row.unread } })
				}
			/>
			{row.snoozable && (
				<SnoozeMenu
					threadId={threadId}
					choices={snooze}
					align="end"
					tabIndex={-1}
				/>
			)}
			<Star messageId={row.messageId} starred={row.starred} tabIndex={-1} />
		</>
	);
}

// Search over the listing shown: the words are sent on Enter; while results show, their count
// and Clear. The field starts from the engine's query, so it is keyed by whether a search shows.
function Search({
	search,
	touch,
}: {
	search: InboxView["search"];
	touch?: boolean;
}) {
	return (
		<SearchField
			key={search.case ?? unset("InboxView.search")}
			search={search}
			touch={touch}
		/>
	);
}

function SearchField({
	search,
	touch,
}: {
	search: InboxView["search"];
	touch?: boolean;
}) {
	const send = useSend();
	const [query, setQuery] = useDraft(
		match(search, "InboxView.search", {
			notSearching: () => "",
			results: (results) => results.query,
		}),
	);
	const clear = () => send({ case: "clearSearch", value: {} });
	const outcome = useOutcome({ case: "form", value: Form.SEARCH });
	return (
		<div className="search">
			<label className={cx("search-box", touch && "touch")}>
				<Icon name="search" className="subtle" />
				<input
					type="search"
					aria-label="Search mail"
					placeholder="Search"
					value={query}
					onChange={(event) => setQuery(event.target.value)}
					onKeyDown={(event) => {
						if (event.key === "Enter")
							send({ case: "searchMail", value: { query } });
						if (event.key === "Escape") clear();
					}}
				/>
				{match(search, "InboxView.search", {
					notSearching: () => null,
					results: () => (
						<IconButton
							icon="close"
							size="sm"
							label="Clear search"
							onClick={clear}
						/>
					),
				})}
			</label>
			{match(search, "InboxView.search", {
				notSearching: () => null,
				results: (results) => (
					<span className="xs subtle settle">{results.count}</span>
				),
			})}
			<OutcomeNote outcome={outcome} />
		</div>
	);
}

function Note({ note }: { note: MailRow["agentNote"] }) {
	return match(note, "MailRow.agent_note", {
		none: () => null,
		noted: (noted) => (
			<span className={cx("trunc xs", noteClass(noted.kind))}>
				{noted.text}
			</span>
		),
	});
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
	const id = match(reading, "InboxView.reading", {
		preview: (preview) => preview.threadId,
		nothing: () => "nothing",
	});
	return (
		<ReadingPane id={id} toolbar={<Toolbar reading={reading} />}>
			<Reading reading={reading} />
		</ReadingPane>
	);
}

// The thread's toolbar with Open thread; nothing selected leaves an empty bar.
function Toolbar({ reading }: { reading: InboxView["reading"] }) {
	const send = useSend();
	switch (reading.case) {
		case "preview": {
			const { threadId } = reading.value;
			return (
				<MailToolbar
					threadId={threadId}
					actions={reading.value.actions}
					place={reading.value.place}
					trail={
						<Button
							variant="ghost"
							onClick={() => send({ case: "openThread", value: { threadId } })}
						>
							Open thread
						</Button>
					}
				/>
			);
		}
		case "nothing":
			return <div className="mail-toolbar" />;
		case undefined:
			return unset("InboxView.reading");
	}
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

// The thread's subject and Scribe's summary over its latest message's card.
function Preview({ preview }: { preview: ReadingPreview }) {
	return (
		<article className="read-article">
			<header className="thread-head">
				<h2 className="h1">{preview.subject}</h2>
				<span className="sm subtle">{preview.toLine}</span>
			</header>
			<Summary summary={preview.scribe} label="Summary by Scribe" />
			<MessageCard
				messageId={preview.messageId}
				from={preview.from}
				recipients={preview.recipients}
				time={preview.time}
				subject={preview.messageSubject}
				starred={preview.starred}
				attachments={preview.attachments}
				invitations={[]}
			>
				<MessageBody url={preview.bodyUrl} title={preview.subject} />
			</MessageCard>
			<Rows label="Agent drafts">
				{preview.drafts.map((draft, i) => (
					<Row key={draft.approvalId} index={i}>
						<AgentDraft draft={draft} />
					</Row>
				))}
			</Rows>
		</article>
	);
}

function PhoneInbox({ view, nav }: ScreenProps<InboxView>) {
	const send = useSend();
	return (
		<div className="inbox-phone">
			<PhoneHead
				bar={
					<>
						<Logo brand={{ kind: "mark" }} size={24} />
						<span className="grow">
							<MailboxMenu nav={nav} title={view.title} />
						</span>
					</>
				}
			>
				<Search search={view.search} touch />
				<UnlessSearching search={view.search}>
					<Categories categories={view.categories} touch />
				</UnlessSearching>
			</PhoneHead>
			<div className="scroll noscroll inbox-phone-scroll">
				<Rows label="Messages">
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
				aria-label="New mail"
				onClick={() => send(newMail)}
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
				<Note note={row.agentNote} />
			</span>
			<span className="xs subtle row-time">{row.time}</span>
		</button>
	);
}
