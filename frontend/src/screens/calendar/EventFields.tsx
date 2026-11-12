import type { MessageInitShape } from "@bufbuild/protobuf";
import { unset, useSend } from "../../engine";
import type { EventDetail } from "../../gen/ui/v1/calendar_pb";
import { Form } from "../../gen/ui/v1/common_pb";
import type { SaveEventSchema } from "../../gen/ui/v1/shell_pb";
import { useDraft } from "../../presentation";
import { Button, IconButton } from "../../ui/Button";
import { Confirm } from "../../ui/Confirm";
import { cx } from "../../ui/cx";
import { Field } from "../../ui/Field";
import { OutcomeNote, pulseOn, useOutcome } from "../../ui/Outcome";
import { Segmented } from "../../ui/Segmented";
import { stagger } from "../../ui/stagger";
import { calendarColor } from "./color";

// The event's fields and footer, in the desktop drawer and the phone sheet. Every field is a
// draft of the user's (input before commit) that starts at the engine's value; Save sends them
// all with Intent.save_event, and a refusal shows under the footer. A saved event offers Delete,
// which asks first; the unsaved new one is dropped by Cancel.
export type Saving = NonNullable<
	MessageInitShape<typeof SaveEventSchema>["event"]
>;

export function EventFields({
	event,
	saving,
}: {
	event: EventDetail;
	saving: Saving;
}) {
	const send = useSend();
	const [title, setTitle] = useDraft(event.title);
	const [day, setDay] = useDraft(event.dayId);
	const [start, setStart] = useDraft(event.startMinute);
	const [end, setEnd] = useDraft(event.endMinute);
	const [calendar, setCalendar] = useDraft(
		(
			event.calendars.find((c) => c.selected) ??
			unset("EventEditor: a calendar selected")
		).id,
	);
	const [guests, setGuests] = useDraft(event.guests);
	const [guest, setGuest] = useDraft();
	const outcome = useOutcome(
		saving.case === "eventId"
			? { case: "eventId", value: saving.value }
			: { case: "form", value: Form.NEW_EVENT },
	);
	// The guest list is input before Save: a typed name joins it as typed; with nothing typed there
	// is nothing to add.
	const addGuest = () => {
		if (guest === "") return;
		setGuests([...guests, guest]);
		setGuest("");
	};
	const section = (index: number) => ({
		className: "cal-field settle",
		style: stagger(index),
	});
	const times = (
		label: string,
		value: number,
		set: (minute: number) => void,
	) => (
		<select
			aria-label={label}
			value={value}
			onChange={(e) => set(Number(e.target.value))}
		>
			{event.times.map((time) => (
				<option key={time.minute} value={time.minute}>
					{time.label}
				</option>
			))}
		</select>
	);
	return (
		<div className="cal-fields">
			<div className="scroll cal-fields-scroll">
				<div {...section(0)}>
					<Field label="Title" value={title} onChange={setTitle} />
				</div>
				<div {...section(1)}>
					<span className="label">When</span>
					<div className="cal-when">
						<select
							aria-label="Day"
							value={day}
							onChange={(e) => setDay(e.target.value)}
						>
							{event.days.map((choice) => (
								<option key={choice.id} value={choice.id}>
									{choice.label}
								</option>
							))}
						</select>
						{times("Starts", start, setStart)}
						{times("Ends", end, setEnd)}
					</div>
				</div>
				<div {...section(2)}>
					<span className="label">Calendar</span>
					<Segmented
						kind="radio"
						label="Calendar"
						segments={event.calendars.map((choice) => ({
							key: choice.id,
							label: choice.name,
							lead: (
								<span
									className="cal-swatch"
									data-cal={calendarColor(choice.color)}
									aria-hidden="true"
								/>
							),
							selected: choice.id === calendar,
							onSelect: () => setCalendar(choice.id),
						}))}
					/>
					<span className="xs subtle">{event.savedTo}</span>
				</div>
				<div {...section(3)}>
					<span className="cal-guest-add">
						<span className="grow">
							<Field
								label="Guests"
								value={guest}
								onChange={setGuest}
								onEnter={addGuest}
								placeholder="Name or address"
							/>
						</span>
						<Button variant="soft" onClick={addGuest}>
							Add
						</Button>
					</span>
					{guests.map((each) => (
						<div key={each} className="cal-guest row-in">
							<span className="trunc grow">{each}</span>
							<IconButton
								icon="close"
								size="sm"
								label={`Remove ${each}`}
								onClick={() => setGuests(guests.filter((g) => g !== each))}
							/>
						</div>
					))}
				</div>
			</div>
			<OutcomeNote outcome={outcome} className="cal-outcome" />
			<div className="cal-foot">
				{saving.case === "eventId" && (
					<Confirm
						question="Delete this event?"
						action="Delete"
						side="above"
						onConfirm={() =>
							send({ case: "deleteEvent", value: { eventId: saving.value } })
						}
						trigger={(props) => (
							<Button {...props} variant="ghost" className="bad">
								Delete
							</Button>
						)}
					/>
				)}
				<span className="grow" />
				<Button
					variant="ghost"
					onClick={() => send({ case: "closeEvent", value: {} })}
				>
					Cancel
				</Button>
				<Button
					variant="primary"
					className={cx("cal-save", pulseOn(outcome))}
					onClick={() =>
						send({
							case: "saveEvent",
							value: {
								event: saving,
								title,
								dayId: day,
								startMinute: start,
								endMinute: end,
								calendarId: calendar,
								guests,
							},
						})
					}
				>
					Save
				</Button>
			</div>
		</div>
	);
}
