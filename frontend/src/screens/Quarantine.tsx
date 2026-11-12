import "./quarantine.css";
import { unset, useSend } from "../engine";
import type {
	QuarantineDetail,
	QuarantineList,
	QuarantineRow,
	QuarantineView,
} from "../gen/ui/v1/quarantine_pb";
import { useLayout } from "../presentation";
import { AllClear } from "../ui/AllClear";
import { Button, IconButton } from "../ui/Button";
import { Confirm } from "../ui/Confirm";
import { Dot } from "../ui/Dot";
import { Icon } from "../ui/Icon";
import { Logo } from "../ui/Logo";
import { ListPane, MailHeader, MailListRow, ReadingPane } from "../ui/MailList";
import { MessageBody } from "../ui/MessageBody";
import { PhoneHead } from "../ui/PhoneHead";
import { Row, Rows } from "../ui/Rows";
import { useSelectionGroup } from "../ui/Selection";
import { SelectList, tabStop } from "../ui/SelectList";
import type { ScreenProps } from "./Screen";

// Mail Security Guard caught or a blocked sender sent. Split layout: the list and the mail beside
// it, where a row click selects. Narrower, each mail is a card with its actions and no body, as
// on the phone.

export function Quarantine({ view }: ScreenProps<QuarantineView>) {
	const layout = useLayout();
	switch (layout) {
		case "phone":
			return <PhoneQuarantine state={view.state} lede={view.lede} />;
		case "list":
			return (
				<ListPane label="Quarantine" head={<Heading lede={view.lede} />}>
					<Cards state={view.state} />
				</ListPane>
			);
		case "split":
			return (
				<>
					<ListPane label="Quarantine" head={<Heading lede={view.lede} />}>
						<List state={view.state} />
					</ListPane>
					<ReadingPane
						id={shownId(selected(view.state))}
						toolbar={
							<div className="mail-toolbar" role="toolbar" aria-label="Mail">
								<span className="grow" />
								<SelectedActions state={view.state} />
							</div>
						}
					>
						<Selected state={view.state} />
					</ReadingPane>
				</>
			);
	}
}

// The mail shown beside the list: none when quarantine is empty.
type Shown = { kind: "none" } | { kind: "mail"; detail: QuarantineDetail };

function selected(state: QuarantineView["state"]): Shown {
	switch (state.case) {
		case "empty":
			return { kind: "none" };
		case "list":
			return {
				kind: "mail",
				detail: state.value.selected ?? unset("QuarantineList.selected"),
			};
		case undefined:
			return unset("QuarantineView.state");
	}
}

function shownId(shown: Shown) {
	switch (shown.kind) {
		case "none":
			return "empty";
		case "mail":
			return shown.detail.id;
	}
}

function Heading({ lede }: { lede: string }) {
	return (
		<>
			<h1 className="h2">Quarantine</h1>
			<span className="xs subtle">{lede}</span>
		</>
	);
}

// Not spam releases the mail to the inbox and takes the sender off the block list; Delete now
// asks first. The mail leaves the list when the engine's view no longer has it.
function Actions({ id, size }: { id: string; size?: "lg" }) {
	const send = useSend();
	return (
		<>
			<Button
				variant="soft"
				size={size}
				onClick={() =>
					send({ case: "releaseQuarantined", value: { quarantinedId: id } })
				}
			>
				Not spam
			</Button>
			<Confirm
				question="Delete this mail now?"
				action="Delete"
				align="end"
				onConfirm={() =>
					send({ case: "deleteQuarantined", value: { quarantinedId: id } })
				}
				trigger={(props) => (
					<Button {...props} variant="ghost" size={size} className="bad">
						Delete now
					</Button>
				)}
			/>
		</>
	);
}

function Empty() {
	return (
		<AllClear
			title="Nothing in quarantine"
			detail="Security Guard tells you when it catches something"
		/>
	);
}

function List({ state }: { state: QuarantineView["state"] }) {
	switch (state.case) {
		case "empty":
			return <Empty />;
		case "list":
			return <DesktopRows list={state.value} />;
		case undefined:
			return unset("QuarantineView.state");
	}
}

function DesktopRows({ list }: { list: QuarantineList }) {
	const send = useSend();
	const group = useSelectionGroup();
	const selected = list.rows.find((row) => row.selected);
	return (
		<SelectList active={selected?.id}>
			<Rows label="Quarantined mail">
				{list.rows.map((row, i) => (
					<Row key={row.id} index={i}>
						<MailListRow
							group={group}
							selected={row.selected}
							tone="err"
							from={row.from}
							provider={row.provider}
							account={row.account}
							time={row.time}
							onSelect={() =>
								send({
									case: "selectQuarantined",
									value: { quarantinedId: row.id },
								})
							}
							tabStop={tabStop(row.selected, i, selected !== undefined)}
						>
							<span className="trunc sm">{row.subject}</span>
							<span className="trunc xs bad">{row.reason}</span>
							<span className="trunc xs subtle">{row.outcome}</span>
						</MailListRow>
					</Row>
				))}
			</Rows>
		</SelectList>
	);
}

function SelectedActions({ state }: { state: QuarantineView["state"] }) {
	const shown = selected(state);
	switch (shown.kind) {
		case "none":
			return null;
		case "mail":
			return <Actions id={shown.detail.id} />;
	}
}

function Selected({ state }: { state: QuarantineView["state"] }) {
	const shown = selected(state);
	switch (shown.kind) {
		case "none":
			return null;
		case "mail":
			return <Mail detail={shown.detail} />;
	}
}

function Mail({ detail }: { detail: QuarantineDetail }) {
	return (
		<article className="read-article">
			<MailHeader
				subject={detail.subject}
				from={detail.from}
				toLine={detail.toLine}
				time={detail.time}
			/>
			<div className="well quarantine-why">
				<Icon name="shield" className="quarantine-shield bad" />
				<span className="quarantine-why-text">
					<span className="xs subtle">{detail.caughtBy}</span>
					<span className="quarantine-reason-text">{detail.why}</span>
					<span className="xs subtle">{detail.outcome}</span>
				</span>
			</div>
			<span className="xs subtle status-line">
				<Icon name="link-off" size="sm" />
				Shown as plain text. Links and images are off
			</span>
			<MessageBody url={detail.bodyUrl} title={detail.subject} />
		</article>
	);
}

// Without the mail beside it, each is a card carrying its own actions.
function Cards({ state }: { state: QuarantineView["state"] }) {
	switch (state.case) {
		case "empty":
			return <Empty />;
		case "list":
			return (
				<>
					<Rows label="Quarantined mail" className="quarantine-card-rows">
						{state.value.rows.map((row, i) => (
							<Row key={row.id} index={i}>
								<Card row={row} />
							</Row>
						))}
					</Rows>
					<span className="xs subtle quarantine-foot">
						Links and images in caught mail stay off
					</span>
				</>
			);
		case undefined:
			return unset("QuarantineView.state");
	}
}

function Card({ row }: { row: QuarantineRow }) {
	return (
		<div className="tile quarantine-card">
			<span className="quarantine-card-head">
				<Dot tone="err" />
				<span className="trunc grow strong">{row.from}</span>
				<Logo
					brand={{ kind: "mail", provider: row.provider }}
					label={row.account}
					size={15}
				/>
				<span className="xs subtle">{row.time}</span>
			</span>
			<span className="quarantine-subject">{row.subject}</span>
			<span className="sm bad">{row.reason}</span>
			<span className="xs subtle">{row.outcome}</span>
			<span className="quarantine-card-actions">
				<Actions id={row.id} size="lg" />
			</span>
		</div>
	);
}

function PhoneQuarantine({
	state,
	lede,
}: {
	state: QuarantineView["state"];
	lede: string;
}) {
	const send = useSend();
	return (
		<div className="quarantine-phone">
			<PhoneHead
				bar={
					<>
						<IconButton
							icon="chevron-left"
							size="lg"
							touch
							label="Back"
							onClick={() => send({ case: "back", value: {} })}
						/>
						<h1 className="h1 grow">Quarantine</h1>
					</>
				}
			>
				<span className="sm subtle">{lede}</span>
			</PhoneHead>
			<div className="scroll noscroll quarantine-cards">
				<Cards state={state} />
			</div>
		</div>
	);
}
