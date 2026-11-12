import { allOf, calendarProviders } from "@everymail/design/brands";
import "./calendar.css";
import { AnimatePresence } from "motion/react";
import { type CSSProperties, Fragment } from "react";
import { match, unset, useSend } from "../engine";
import type {
	CalendarDay,
	CalendarEvent,
	CalendarView,
	EventDetail,
	MonthPeriod,
	SelectedInvitation,
	WeekPeriod,
	YearPeriod,
} from "../gen/ui/v1/calendar_pb";
import type { InvitationAnswer } from "../gen/ui/v1/common_pb";
import { useLayout } from "../presentation";
import { Button, IconButton } from "../ui/Button";
import { cx } from "../ui/cx";
import { Dot } from "../ui/Dot";
import { Icon } from "../ui/Icon";
import { Invitation } from "../ui/Invitation";
import { Logo } from "../ui/Logo";
import { Overlap } from "../ui/Overlap";
import { PresentDrawer, PresentPopover, PresentSheet } from "../ui/Overlay";
import { PhoneHead } from "../ui/PhoneHead";
import { Row, Rows } from "../ui/Rows";
import { Segmented } from "../ui/Segmented";
import { SelectionMark, useSelectionGroup } from "../ui/Selection";
import { Switch } from "../ui/Switch";
import { stagger } from "../ui/stagger";
import { calendarColor } from "./calendar/color";
import { EventFields, type Saving } from "./calendar/EventFields";
import type { ScreenProps } from "./Screen";

// Every calendar in a week, a month or a year. Selecting an event, closing it, picking a day or a
// month, the view and previous, next and today are the engine's intents; the engine opens the event
// drawer (desktop) or sheet (phone) through `selected`; New event adds one on the selected day and
// opens it there. Geometry is the one place
// numbers are used: events and hour marks carry minutes, which CSS turns into a place and a height.
export function Calendar({ view }: ScreenProps<CalendarView>) {
	const layout = useLayout();
	if (layout === "phone") return <PhoneCalendar view={view} />;
	return (
		<>
			<section className="panel cal-main" aria-label="Calendar">
				<Bar view={view} />
				<Overlap id={periodKey(view.period)} className="cal-period">
					<Period view={view} />
				</Overlap>
			</section>
			{layout === "split" && <Side view={view} />}
			<Drawer selected={view.selected} />
		</>
	);
}

function periodKey(period: CalendarView["period"]) {
	return period.case ?? unset("CalendarView.period");
}

function Period({ view }: { view: CalendarView }) {
	const { period } = view;
	switch (period.case) {
		case "week":
			return <Week week={period.value} selected={view.selected} />;
		case "month":
			return <Month month={period.value} />;
		case "year":
			return <Year year={period.value} />;
		case undefined:
			return unset("CalendarView.period");
	}
}

// The title, the view (Week, Month, Year), previous, today and next, and New event.
function Bar({ view }: { view: CalendarView }) {
	const send = useSend();
	const phone = useLayout() === "phone";
	const range = match(view.period, "CalendarView.period", {
		week: (week) => week.range,
		month: () => "",
		year: () => "",
	});
	return (
		<div className="cal-bar">
			{!phone && (
				<span className="grow cal-title-line">
					<h1 className="h2">{view.title}</h1>
					<span className="sm subtle">{range}</span>
				</span>
			)}
			<Segmented
				kind="tabs"
				label="Calendar view"
				touch={phone}
				className="cal-modes"
				segments={view.modes.map((tab) => ({
					key: String(tab.mode),
					label: tab.label,
					selected: tab.current,
					onSelect: () =>
						send({ case: "selectCalendarView", value: { mode: tab.mode } }),
				}))}
			/>
			<span className="cal-nav">
				<IconButton
					icon="chevron-left"
					label="Previous"
					touch={phone}
					onClick={() => send({ case: "showPreviousPeriod", value: {} })}
				/>
				<Button
					variant="ghost"
					onClick={() => send({ case: "showToday", value: {} })}
				>
					Today
				</Button>
				<IconButton
					icon="chevron-right"
					label="Next"
					touch={phone}
					onClick={() => send({ case: "showNextPeriod", value: {} })}
				/>
			</span>
			{!phone && (
				<Button
					variant="primary"
					onClick={() => send({ case: "newEvent", value: {} })}
				>
					<Icon name="plus" />
					New event
				</Button>
			)}
		</div>
	);
}

// What the drawer or sheet shows: an event (saved, or the unsaved new one), or an invitation to
// answer, with the day it is on.
function chosen(selected: CalendarView["selected"]): {
	event?: { detail: EventDetail; saving: Saving };
	invitation?: SelectedInvitation;
} {
	switch (selected.case) {
		case "none":
			return {};
		case "event":
			return {
				event: {
					detail: selected.value,
					saving: { case: "eventId", value: selected.value.id },
				},
			};
		case "unsaved":
			return {
				event: {
					detail: selected.value,
					saving: { case: "unsaved", value: {} },
				},
			};
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

// Monday to Sunday on an hour grid, a hairline between the days. A day's head selects it.
function Week({
	week,
	selected,
}: {
	week: WeekPeriod;
	selected: CalendarView["selected"];
}) {
	const send = useSend();
	const { invitation } = chosen(selected);
	const first = (week.hours[0] ?? unset("WeekPeriod.hours")).minute;
	const last = (week.hours.at(-1) ?? unset("WeekPeriod.hours")).minute;
	return (
		<div
			className="cal-week"
			style={
				{
					"--first": first,
					"--last": last,
					"--days": week.days.length,
				} as CSSProperties
			}
		>
			<div className="cal-cols cal-heads">
				<span />
				{week.days.map((day) => (
					<button
						key={day.id}
						type="button"
						className={cx(
							"press cal-head",
							day.today && "today",
							day.selected && "on",
						)}
						aria-current={day.today ? "date" : undefined}
						aria-pressed={day.selected}
						onClick={() =>
							send({ case: "selectDay", value: { dayId: day.id } })
						}
					>
						<span className="xs cal-weekday">{day.weekday}</span>
						<span className="cal-daynum">{day.day}</span>
					</button>
				))}
			</div>
			<div className="scroll cal-scroll">
				<div className="cal-cols cal-body">
					{week.hours.map((hour) => (
						<span
							key={hour.minute}
							className="cal-line"
							aria-hidden="true"
							style={{ "--at": hour.minute } as CSSProperties}
						/>
					))}
					<div className="cal-hours">
						{week.hours.map((hour) => (
							<span
								key={hour.minute}
								className="xs subtle cal-hour"
								style={{ "--at": hour.minute } as CSSProperties}
							>
								{hour.label}
							</span>
						))}
					</div>
					{week.days.map((day, i) => (
						<DayColumn
							key={day.id}
							day={day}
							index={i}
							asking={invitation?.dayId === day.id ? invitation : undefined}
						/>
					))}
				</div>
			</div>
		</div>
	);
}

// The weeks of the month, Monday first: each day with up to three chips, or two and "+N more".
// Selecting a day opens its week.
function Month({ month }: { month: MonthPeriod }) {
	const send = useSend();
	return (
		<div className="cal-month">
			<div className="cal-month-row cal-month-heads" aria-hidden="true">
				{month.weekdays.map((weekday) => (
					<span key={weekday} className="xs subtle">
						{weekday}
					</span>
				))}
			</div>
			{month.weeks.map((week) => (
				<div key={week.days[0]?.id} className="cal-month-row cal-month-week">
					{week.days.map((day, i) => (
						<button
							key={day.id}
							type="button"
							className={cx(
								"press cal-month-day settle",
								!day.inMonth && "out",
								day.today && "today",
								day.selected && "on",
							)}
							style={stagger(i)}
							aria-current={day.today ? "date" : undefined}
							aria-pressed={day.selected}
							onClick={() =>
								send({ case: "selectDay", value: { dayId: day.id } })
							}
						>
							<span className="cal-month-num">{day.day}</span>
							{day.chips.map((chip) => (
								<span
									key={chip.id}
									className="trunc cal-chip"
									data-cal={calendarColor(chip.color)}
								>
									<span className="cal-chip-time">{chip.start}</span>{" "}
									{chip.title}
								</span>
							))}
							{day.more && <span className="xs subtle">{day.more}</span>}
						</button>
					))}
				</div>
			))}
		</div>
	);
}

// Twelve small months; the days of neighbouring months are left blank. Selecting a month opens it.
function Year({ year }: { year: YearPeriod }) {
	const send = useSend();
	return (
		<div className="cal-year">
			{year.months.map((month, i) => (
				<button
					key={month.id}
					type="button"
					className={cx("press cal-year-month settle", month.current && "on")}
					style={stagger(i)}
					aria-label={month.name}
					aria-current={month.current ? "date" : undefined}
					onClick={() =>
						send({ case: "selectMonth", value: { monthId: month.id } })
					}
				>
					<span className="strong sm">{month.name}</span>
					<span className="cal-year-grid" aria-hidden="true">
						{year.weekdays.map((weekday, j) => (
							<span
								// biome-ignore lint/suspicious/noArrayIndexKey: weekday letters repeat ("S", "S")
								key={j}
								className="subtle"
							>
								{weekday}
							</span>
						))}
						{month.weeks.flatMap((week) =>
							week.days.map((day) => (
								<span
									key={day.id}
									className={cx(
										!day.inMonth && "out",
										day.today && "today",
										day.busy && "busy",
									)}
								>
									{day.inMonth ? day.day : ""}
								</span>
							)),
						)}
					</span>
				</button>
			))}
		</div>
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
					label={event.detail.heading}
					onClose={close}
					className="cal-drawer"
				>
					<div className="cal-panel">
						<div className="cal-drawer-head">
							<h2 className="h2 grow">{event.detail.heading}</h2>
							<IconButton icon="close" label="Close" onClick={close} />
						</div>
						<Overlap id={event.detail.id} className="cal-overlap">
							<EventFields event={event.detail} saving={event.saving} />
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
	// The invitation selected on this day, if any: its popover opens by its event.
	asking: SelectedInvitation | undefined;
}) {
	const send = useSend();
	const intent = useEventIntent();
	const events = eventsOf(day);
	return (
		<div className="cal-day">
			{events.map((event, j) => {
				const look = lookOf(event);
				const answering =
					asking?.invitation?.eventId === event.id
						? asking.invitation
						: undefined;
				return (
					<Fragment key={event.id}>
						<button
							type="button"
							className="cal-event press settle"
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
						<AnimatePresence>
							{answering && (
								<PresentPopover
									key="invitation"
									label="Reply to invitation"
									keep=".cal-pop, .cal-event"
									className="cal-pop"
									style={{ "--end": event.endMinute } as CSSProperties}
									onClose={() => send({ case: "closeEvent", value: {} })}
								>
									<Invitation invitation={answering} form="thread" />
								</PresentPopover>
							)}
						</AnimatePresence>
					</Fragment>
				);
			})}
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
						className="cal-source row-in"
						style={stagger(i)}
						data-cal={calendarColor(calendar.color)}
					>
						<Logo brand={{ kind: "calendar", provider: calendar.provider }} />
						<span className="cal-swatch" aria-hidden="true" />
						<span className="grow two-line">
							<span>{calendar.name}</span>
							<span className="xs subtle">{calendar.providerName}</span>
						</span>
						<Switch
							on={calendar.shown}
							label={calendar.name}
							onToggle={() =>
								send({
									case: "setCalendarShown",
									value: { calendarId: calendar.id, shown: !calendar.shown },
								})
							}
						/>
					</div>
				))}
				<span className="xs subtle cal-note">
					{allOf([...calendarProviders.direct, "CalDAV"])} calendars such as{" "}
					{allOf(calendarProviders.caldav)} join when you{" "}
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

// The phone: the title and the view bar on top; a week is its day strip over the selected day's
// agenda, a month and a year their grids.
function PhoneCalendar({ view }: { view: CalendarView }) {
	const send = useSend();
	const { event, invitation } = chosen(view.selected);
	const close = () => send({ case: "closeEvent", value: {} });
	const { period } = view;
	// Month and year scroll as a page; the week lays out its own scrolling.
	const scrolling = () => (
		<div className="scroll noscroll cal-phone-scroll">
			<Period view={view} />
		</div>
	);
	return (
		<div className="cal-phone">
			<PhoneHead
				bar={
					<>
						<h1 className="h1 grow cal-phone-title">{view.title}</h1>
						<IconButton
							icon="plus"
							label="New event"
							touch
							onClick={() => send({ case: "newEvent", value: {} })}
						/>
					</>
				}
			>
				<Bar view={view} />
			</PhoneHead>
			<Overlap id={periodKey(period)} className="cal-period">
				{match(period, "CalendarView.period", {
					week: (week) => <PhoneWeek week={week} />,
					month: scrolling,
					year: scrolling,
				})}
			</Overlap>
			<AnimatePresence>
				{(event || invitation) && (
					<PresentSheet
						key="sheet"
						label={event ? event.detail.heading : "Reply to invitation"}
						onDismiss={close}
					>
						{event && (
							<EventFields event={event.detail} saving={event.saving} />
						)}
						{invitation && (
							<Invitation
								invitation={
									invitation.invitation ??
									unset("SelectedInvitation.invitation")
								}
								form="sheet"
							/>
						)}
					</PresentSheet>
				)}
			</AnimatePresence>
		</div>
	);
}

function PhoneWeek({ week }: { week: WeekPeriod }) {
	const send = useSend();
	const day =
		week.days.find((d) => d.selected) ?? unset("CalendarDay.selected");
	const group = useSelectionGroup();
	return (
		<>
			<div
				className="cal-strip"
				role="tablist"
				aria-label="Day"
				style={{ "--days": week.days.length } as CSSProperties}
			>
				{week.days.map((d) => (
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
						onClick={() => send({ case: "selectDay", value: { dayId: d.id } })}
					>
						{d.selected && <SelectionMark group={group} />}
						<span className="xs cal-weekday">{d.weekday}</span>
						<span className="cal-daynum">{d.day}</span>
					</button>
				))}
			</div>
			<div className="scroll noscroll cal-agenda">
				<Overlap id={day.id}>
					<span className="label cal-agenda-label">{day.title}</span>
					<Agenda agenda={day.agenda} />
				</Overlap>
			</div>
		</>
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
