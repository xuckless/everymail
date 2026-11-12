import "./quarantine.css";
import { unset, useSend } from "../engine";
import type {
	QuarantineDetail,
	QuarantineList,
	QuarantineRow,
	QuarantineView,
} from "../gen/ui/v1/quarantine_pb";
import { useLayout } from "../presentation";
import { Button, IconButton } from "../ui/Button";
import { Dot } from "../ui/Dot";
import { Icon } from "../ui/Icon";
import { Logo } from "../ui/Logo";
import { ListPane, MailHeader, MailListRow, ReadingPane } from "../ui/MailList";
import { MessageBody } from "../ui/MessageBody";
import { Row, Rows } from "../ui/Rows";
import { useSelectionGroup } from "../ui/Selection";
import { SelectList, useOptionIds } from "../ui/SelectList";
import type { ScreenProps } from "./Screen";

// Mail Security Guard caught or a blocked sender sent. Split layout: the list and the mail beside
// it, where a row click selects. Narrower, each mail is a card with its actions and no body, as
// on the phone. Not spam and Delete now have no intent yet and are drawn without a handler.
const caption =
	"Caught by Security Guard or sent by a blocked sender. Each copy is dropped 30 days after it lands here";

export function Quarantine({ view }: ScreenProps<QuarantineView>) {
	const layout = useLayout();
	switch (layout) {
		case "phone":
			return <PhoneQuarantine state={view.state} />;
		case "list":
			return (
				<ListPane label="Quarantine" head={<Heading />}>
					<Cards state={view.state} />
				</ListPane>
			);
		case "split":
			return (
				<>
					<ListPane label="Quarantine" head={<Heading />}>
						<List state={view.state} />
					</ListPane>
					<ReadingPane
						id={selected(view.state)?.id ?? "empty"}
						bar={<Actions disabled={view.state.case === "empty"} />}
					>
						<Selected state={view.state} />
					</ReadingPane>
				</>
			);
	}
}

function selected(state: QuarantineView["state"]) {
	switch (state.case) {
		case "empty":
			return undefined;
		case "list":
			return state.value.selected ?? unset("QuarantineList.selected");
		case undefined:
			return unset("QuarantineView.state");
	}
}

function Heading() {
	return (
		<>
			<h1 className="h2">Quarantine</h1>
			<span className="xs subtle">{caption}</span>
		</>
	);
}

function Actions({ disabled }: { disabled?: boolean }) {
	return (
		<>
			<Button variant="soft" disabled={disabled}>
				Not spam
			</Button>
			<Button variant="ghost" className="quarantine-delete" disabled={disabled}>
				Delete now
			</Button>
		</>
	);
}

function Empty() {
	return (
		<div className="quarantine-empty settle">
			<Icon name="check" size="lg" className="ok" />
			<span className="strong">Nothing in quarantine</span>
			<span className="sm subtle">
				Security Guard tells you when it catches something
			</span>
		</div>
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
	const optionId = useOptionIds();
	const selected = list.rows.find((row) => row.selected);
	return (
		<SelectList
			label="Quarantined mail"
			active={selected && optionId(selected.id)}
		>
			<Rows label="Quarantined mail" group>
				{list.rows.map((row, i) => (
					<Row
						key={row.id}
						index={i}
						option={{ id: optionId(row.id), selected: row.selected }}
					>
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
						>
							<span className="trunc sm">{row.subject}</span>
							<span className="trunc xs quarantine-reason">{row.reason}</span>
							<span className="trunc xs subtle">{row.outcome}</span>
						</MailListRow>
					</Row>
				))}
			</Rows>
		</SelectList>
	);
}

function Selected({ state }: { state: QuarantineView["state"] }) {
	const detail = selected(state);
	return detail ? <Mail detail={detail} /> : null;
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
				<Icon name="shield" className="quarantine-shield" />
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
			<span className="sm quarantine-reason">{row.reason}</span>
			<span className="xs subtle">{row.outcome}</span>
			<span className="quarantine-card-actions">
				<Actions />
			</span>
		</div>
	);
}

function PhoneQuarantine({ state }: { state: QuarantineView["state"] }) {
	const send = useSend();
	return (
		<div className="quarantine-phone">
			<header className="quarantine-phone-head">
				<div className="quarantine-phone-bar">
					<IconButton
						icon="chevron-left"
						size="lg"
						touch
						label="Back"
						onClick={() => send({ case: "back", value: {} })}
					/>
					<h1 className="h1 grow quarantine-phone-title">Quarantine</h1>
				</div>
				<span className="sm subtle">{caption}</span>
			</header>
			<div className="scroll noscroll quarantine-cards">
				<Cards state={state} />
			</div>
		</div>
	);
}
