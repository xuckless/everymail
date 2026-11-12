import type { ReactNode } from "react";
import type { MailProvider } from "../gen/ui/v1/common_pb";
import { cx } from "./cx";
import { Dot, type Tone } from "./Dot";
import { Logo } from "./Logo";
import { Overlap } from "./Overlap";
import { SelectionMark } from "./Selection";

// The list and reading pane shared by the inbox and Quarantine.

// The list's panel: a head (title, tabs) over its scrolling rows.
export function ListPane({
	label,
	head,
	children,
}: {
	label: string;
	head: ReactNode;
	children: ReactNode;
}) {
	return (
		<section className="panel list-pane" aria-label={label}>
			<div className="list-head">{head}</div>
			<div className="scroll list-scroll">{children}</div>
		</section>
	);
}

// One mail in a list: its dot, the sender with the account's logo, the `lines` under it, the time
// and the hover `actions`. The selected row carries the list's highlight (`group`).
export function MailListRow({
	group,
	selected,
	tone,
	from,
	unread,
	provider,
	account,
	time,
	onSelect,
	actions,
	children,
}: {
	group: string;
	selected: boolean;
	tone: Tone;
	from: string;
	unread?: boolean;
	provider: MailProvider;
	account: string;
	time: string;
	onSelect: () => void;
	actions?: ReactNode;
	children: ReactNode;
}) {
	return (
		<div className="mrow selectable list-row">
			{selected && <SelectionMark group={group} />}
			<Dot tone={tone} className="row-dot" />
			<button
				type="button"
				className="row-main"
				data-select
				aria-current={selected ? "true" : undefined}
				onClick={onSelect}
			>
				<span className="row-from">
					<span className={cx("trunc", unread && "unread")}>{from}</span>
					<Logo brand={{ kind: "mail", provider }} label={account} size={15} />
				</span>
				{children}
			</button>
			<span className="xs subtle row-time">{time}</span>
			{actions && <span className="acts">{actions}</span>}
		</div>
	);
}

// The reading pane: its toolbar, then the selected mail, old and new overlapping (keyed by `id`).
export function ReadingPane({
	id,
	bar,
	children,
}: {
	id: string;
	bar: ReactNode;
	children: ReactNode;
}) {
	return (
		<section className="panel read-pane" aria-label="Reading pane">
			<div className="read-bar">
				<span className="grow" />
				{bar}
			</div>
			<div className="scroll read-scroll">
				<Overlap id={id} className="read-stack">
					{children}
				</Overlap>
			</div>
		</section>
	);
}

// A mail's head: the subject, then who sent it to which account, and when.
export function MailHeader({
	subject,
	from,
	toLine,
	time,
}: {
	subject: string;
	from: string;
	toLine: string;
	time: string;
}) {
	return (
		<header className="mail-header">
			<h2 className="h1">{subject}</h2>
			<div className="mail-from">
				<span className="two-line grow">
					<span className="sender">{from}</span>
					<span className="xs subtle">{toLine}</span>
				</span>
				<span className="xs subtle">{time}</span>
			</div>
		</header>
	);
}
