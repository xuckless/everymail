import "./calendar.css";
import { AnimatePresence } from "motion/react";
import type { CSSProperties } from "react";
import { unset, useSend } from "../engine";
import type {
	CalendarDay,
	CalendarEvent,
	CalendarView,
	EventDetail,
} from "../gen/ui/v1/calendar_pb";
import type {
	InvitationAnswer,
	Invitation as InvitationView,
} from "../gen/ui/v1/common_pb";
import { useLayout } from "../presentation";
import { Button, IconButton } from "../ui/Button";
import { cx } from "../ui/cx";
import { Dot } from "../ui/Dot";
import { Icon } from "../ui/Icon";
import { Invitation } from "../ui/Invitation";
import { Logo } from "../ui/Logo";
import { Overlap } from "../ui/Overlap";
import { PresentDrawer, PresentPopover, PresentSheet } from "../ui/Overlay";
import { Row, Rows } from "../ui/Rows";
import { SelectionMark, useSelectionGroup } from "../ui/Selection";
import { Switch } from "../ui/Switch";
import { stagger } from "../ui/stagger";
import { calendarColor } from "./calendar/color";
import { EventFields } from "./calendar/EventFields";
import type { ScreenProps } from "./Screen";

// One week from every calendar. Selecting an event, closing it and picking a day are the engine's
// intents; the engine opens the event drawer (desktop) or sheet (phone) through `selected`.
// Previous, Today, Next, New event, the calendar switches and the invitation answers have no
// intent yet and are drawn without a handler. Geometry is the one place numbers are used: events
// and hour marks carry minutes, which CSS turns into a place and a height.
export function Calendar({ view }: ScreenProps<CalendarView>) {
	const layout = useLayout();
	switch (layout) {
		case "phone":
			return <PhoneCalendar view={view} />;
		case "list":
			return (
				<>
					<Week view={view} />
					<Drawer selected={view.selected} />
				</>
			);
		case "split":
			return (
				<>
					<Week view={view} />
					<Side view={view} />
					<Drawer selected={view.selected} />
				</>
			);
	}
}

function chosen(selected: CalendarView["selected"]): {
	event?: EventDetail;
	invitation?: InvitationView;
} {
	switch (selected.case) {
		case "none":
			return {};
		case "event":
			return { event: selected.value };
		case "invitation":
			return { invitation: selected.value };
		case undefined:
			return unset("CalendarView.selected");
	}
}

// What the event's kind asks of its look: a pending invitation is raised with a warning dot, a
// declined one is dimmed, the rest wear their calendar's colour.
type Look = "plain" | "pending" | "declined" | "answered";

function lookOf(event: CalendarEvent): Look {
	switch (event.kind.case) {
		case "plain":
			return "plain";
		case "invitation":
			return answerLook(event.kind.value);
		case undefined:
			return unset("CalendarEvent.kind");
	}
}

function answerLook(answer: InvitationAnswer): Look {
	switch (answer.state.case) {
		case "unanswered":
			return "pending";
		case "declined":
			return "declined";
		case "accepted":
		case "tentative":
			return "answered";
		case undefined:
			return unset("InvitationAnswer");
	}
}

// The day's events; a day with nothing planned has none.
function eventsOf(day: CalendarDay): CalendarEvent[] {
	switch (day.agenda.case) {
		case "events":
			return day.agenda.value.events;
		case "nothing":
			return [];
		case undefined:
			return unset("CalendarDay.agenda");
	}
}

function useEventIntent() {
	const send = useSend();
	return (event: CalendarEvent) => () =>
		send(
			event.selected
				? { case: "closeEvent", value: {} }
				: { case: "selectEvent", value: { eventId: event.id } },
		);
}

// An event's second line, raised with a warning dot while its invitation waits.
function Detail({ look, text }: { look: Look; text: string }) {
	return (
		<>
			{look === "pending" && <Dot tone="wait" />}
			{text}
		</>
	);
}

function Week({ view }: { view: CalendarView }) {
	const { invitation } = chosen(view.selected);
	const first = view.hours[0]?.minute ?? 0;
	const last = view.hours.at(-1)?.minute ?? 0;
	return (
		<section
			className="panel cal-week"
			aria-label="Week"
			style={
				{
					"--first": first,
					"--last": last,
					"--days": view.days.length,
				} as CSSProperties
			}
		>
			<div className="cal-bar">
				<h1 className="h2">{view.month}</h1>
				<span className="sm subtle grow cal-range">{view.range}</span>
				<IconButton icon="chevron-left" label="Previous week" />
				<Button variant="ghost">Today</Button>
				<IconButton icon="chevron-right" label="Next week" />
				<Button variant="primary" className="cal-new">
					<Icon name="plus" />
					New event
				</Button>
			</div>
			<div className="cal-cols cal-heads">
				<span />
				{view.days.map((day) => (
					<span
						key={day.id}
						className={cx(
							"cal-head",
							day.today && "today",
							day.selected && "on",
						)}
						aria-current={day.today ? "date" : undefined}
					>
						<span className="xs cal-weekday">{day.weekday}</span>
						<span className="cal-daynum">{day.day}</span>
					</span>
				))}
			</div>
			<div className="scroll cal-scroll">
				<div className="cal-cols cal-body">
					{view.hours.map((hour) => (
						<span
							key={hour.minute}
							className="cal-line"
							aria-hidden="true"
							style={{ "--at": hour.minute } as CSSProperties}
						/>
					))}
					<div className="cal-hours">
						{view.hours.map((hour) => (
							<span
								key={hour.minute}
								className="xs subtle cal-hour"
								style={{ "--at": hour.minute } as CSSProperties}
							>
								{hour.label}
							</span>
						))}
					</div>
					{view.days.map((day, i) => (
						<DayColumn key={day.id} day={day} index={i} asking={invitation} />
					))}
				</div>
			</div>
		</section>
	);
}

function Drawer({ selected }: { selected: CalendarView["selected"] }) {
	const send = useSend();
	const { event } = chosen(selected);
	const close = () => send({ case: "closeEvent", value: {} });
	return (
		<AnimatePresence>
			{event && (
				<PresentDrawer
					key="event"
					label={event.heading}
					onClose={close}
					className="cal-drawer"
				>
					<div className="cal-panel">
						<div className="cal-drawer-head">
							<h2 className="h2 grow">{event.heading}</h2>
							<IconButton icon="close" label="Close" onClick={close} />
						</div>
						<Overlap id={event.id} className="cal-overlap">
							<EventFields event={event} />
						</Overlap>
					</div>
				</PresentDrawer>
			)}
		</AnimatePresence>
	);
}

function DayColumn({
	day,
	index,
	asking,
}: {
	day: CalendarDay;
	index: number;
	asking: InvitationView | undefined;
}) {
	const send = useSend();
	const intent = useEventIntent();
	const events = eventsOf(day);
	const open = events.find((e) => e.selected && e.kind.case === "invitation");
	return (
		<div className="cal-day">
			{events.map((event, j) => {
				const look = lookOf(event);
				return (
					<button
						key={event.id}
						type="button"
						className="cal-event press settle stagger"
						data-cal={calendarColor(event.color)}
						data-look={look}
						aria-haspopup="dialog"
						aria-expanded={event.selected}
						style={
							{
								...stagger(index + j),
								"--start": event.startMinute,
								"--end": event.endMinute,
							} as CSSProperties
						}
						onClick={intent(event)}
					>
						<span className="trunc cal-title">{event.title}</span>
						<span className="trunc cal-sub">
							<Detail look={look} text={event.detail} />
						</span>
					</button>
				);
			})}
			<AnimatePresence>
				{open && asking && (
					<PresentPopover
						key="invitation"
						label="Reply to invitation"
						keep=".cal-pop, .cal-event"
						className="cal-pop"
						style={{ "--end": open.endMinute } as CSSProperties}
						onClose={() => send({ case: "closeEvent", value: {} })}
					>
						<Invitation invitation={asking} form="thread" />
					</PresentPopover>
				)}
			</AnimatePresence>
		</div>
	);
}

function Side({ view }: { view: CalendarView }) {
	const send = useSend();
	return (
		<aside className="cal-side panel" aria-label="Calendars">
			<div className="cal-block">
				<h2 className="h2 cal-side-title">Calendars</h2>
				{view.calendars.map((calendar, i) => (
					<div
						key={calendar.id}
						className="cal-source row-in stagger"
						style={stagger(i)}
						data-cal={calendarColor(calendar.color)}
					>
						<Logo brand={{ kind: "calendar", provider: calendar.provider }} />
						<span className="cal-swatch" aria-hidden="true" />
						<span className="grow two-line">
							<span>{calendar.name}</span>
							<span className="xs subtle">{calendar.providerName}</span>
						</span>
						<Switch on={calendar.shown} label={calendar.name} />
					</div>
				))}
				<span className="xs subtle cal-note">
					Google, Outlook and CalDAV calendars such as iCloud and Fastmail join
					when you{" "}
					<button
						type="button"
						className="link"
						onClick={() => send({ case: "openConnect", value: {} })}
					>
						connect the account
					</button>
				</span>
			</div>
			<div className="cal-block">
				<span className="label">Invitations</span>
				{view.invitations.map((invitation) => (
					<Invitation
						key={invitation.eventId}
						invitation={invitation}
						form="card"
					/>
				))}
			</div>
		</aside>
	);
}

function PhoneCalendar({ view }: { view: CalendarView }) {
	const send = useSend();
	const { event, invitation } = chosen(view.selected);
	const day =
		view.days.find((d) => d.selected) ?? unset("CalendarDay.selected");
	const group = useSelectionGroup();
	const close = () => send({ case: "closeEvent", value: {} });
	return (
		<div className="cal-phone">
			<header className="phone-head">
				<div className="phone-bar">
					<Logo brand={{ kind: "mark" }} size={24} />
					<h1 className="h1 grow cal-phone-title">Calendar</h1>
					<Button variant="soft" className="cal-today">
						Today
					</Button>
				</div>
				<div
					className="cal-strip"
					role="tablist"
					aria-label="Day"
					style={{ "--days": view.days.length } as CSSProperties}
				>
					{view.days.map((d) => (
						<button
							key={d.id}
							type="button"
							role="tab"
							aria-selected={d.selected}
							aria-current={d.today ? "date" : undefined}
							className={cx(
								"cal-strip-day press selectable",
								d.today && "today",
								d.selected && "on",
							)}
							onClick={() =>
								send({ case: "selectDay", value: { dayId: d.id } })
							}
						>
							{d.selected && <SelectionMark group={group} />}
							<span className="xs cal-weekday">{d.weekday}</span>
							<span className="cal-daynum">{d.day}</span>
						</button>
					))}
				</div>
			</header>
			<div className="scroll noscroll cal-agenda">
				<Overlap id={day.id} className="cal-agenda-stack">
					<span className="label cal-agenda-label">{day.title}</span>
					<Agenda agenda={day.agenda} />
				</Overlap>
			</div>
			<AnimatePresence>
				{(event || invitation) && (
					<PresentSheet
						key="sheet"
						label={event ? event.heading : "Reply to invitation"}
						onDismiss={close}
					>
						{event && <EventFields event={event} />}
						{invitation && <Invitation invitation={invitation} form="sheet" />}
					</PresentSheet>
				)}
			</AnimatePresence>
		</div>
	);
}

function Agenda({ agenda }: { agenda: CalendarDay["agenda"] }) {
	const intent = useEventIntent();
	switch (agenda.case) {
		case "events":
			return (
				<Rows label="Events" className="cal-agenda-rows">
					{agenda.value.events.map((e, i) => (
						<Row key={e.id} index={i}>
							<AgendaRow event={e} onSelect={intent(e)} />
						</Row>
					))}
				</Rows>
			);
		case "nothing":
			return (
				<p className="sm subtle settle cal-nothing">{agenda.value.text}</p>
			);
		case undefined:
			return unset("CalendarDay.agenda");
	}
}

// A plain event is only read on the phone; an invitation opens its sheet.
function AgendaRow({
	event,
	onSelect,
}: {
	event: CalendarEvent;
	onSelect: () => void;
}) {
	const look = lookOf(event);
	const body = (
		<>
			<span className="sm cal-start">{event.start}</span>
			<span className="grow two-line">
				<span className="trunc cal-title">{event.title}</span>
				<span className="trunc xs cal-sub">
					<Detail look={look} text={event.agendaLine} />
				</span>
			</span>
		</>
	);
	const props = {
		className: "cal-agenda-row",
		"data-cal": calendarColor(event.color),
		"data-look": look,
	};
	return look === "plain" ? (
		<div {...props}>{body}</div>
	) : (
		<button
			type="button"
			{...props}
			className={cx(props.className, "press")}
			aria-haspopup="dialog"
			onClick={onSelect}
		>
			{body}
		</button>
	);
}
