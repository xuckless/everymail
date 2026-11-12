import { useSend } from "../../engine";
import type { EventDetail } from "../../gen/ui/v1/calendar_pb";
import { useDraft } from "../../presentation";
import { Button, IconButton } from "../../ui/Button";
import { Field } from "../../ui/Field";
import { Segmented } from "../../ui/Segmented";
import { stagger } from "../../ui/stagger";
import { calendarColor } from "./color";

// The event's fields and footer, in the desktop drawer and the phone sheet. Nothing here has an
// intent yet but Cancel: every field keeps what is typed in a draft (input before commit) that
// starts at the engine's value, and Add, Remove, Delete and Save are drawn without a handler.
export function EventFields({ event }: { event: EventDetail }) {
	const send = useSend();
	const [title, setTitle] = useDraft(event.title);
	const [day, setDay] = useDraft(event.days.find((d) => d.selected)?.id);
	const [starts, setStarts] = useDraft(event.starts);
	const [ends, setEnds] = useDraft(event.ends);
	const [calendar, setCalendar] = useDraft(
		event.calendars.find((c) => c.selected)?.id,
	);
	const [guest, setGuest] = useDraft();
	const section = (index: number) => ({
		className: "cal-field settle stagger",
		style: stagger(index),
	});
	return (
		<div className="cal-fields">
			<div className="scroll cal-fields-scroll">
				<div {...section(0)}>
					<Field label="Title" value={title} onChange={setTitle} />
				</div>
				<div {...section(1)}>
					<span className="label">When</span>
					<div className="cal-when">
						<label className="sr-only" htmlFor="cal-day">
							Day
						</label>
						<select
							id="cal-day"
							value={day}
							onChange={(e) => setDay(e.target.value)}
						>
							{event.days.map((choice) => (
								<option key={choice.id} value={choice.id}>
									{choice.label}
								</option>
							))}
						</select>
						<Field
							label="Starts"
							hideLabel
							value={starts}
							onChange={setStarts}
						/>
						<Field label="Ends" hideLabel value={ends} onChange={setEnds} />
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
								placeholder="Name or address"
							/>
						</span>
						<Button variant="soft">Add</Button>
					</span>
					{event.guests.map((guest) => (
						<div key={guest} className="cal-guest">
							<span className="trunc grow">{guest}</span>
							<IconButton icon="close" size="sm" label={`Remove ${guest}`} />
						</div>
					))}
				</div>
			</div>
			<div className="cal-foot">
				<Button variant="ghost" className="cal-delete">
					Delete
				</Button>
				<span className="grow" />
				<Button
					variant="ghost"
					onClick={() => send({ case: "closeEvent", value: {} })}
				>
					Cancel
				</Button>
				<Button variant="primary" className="cal-save">
					Save
				</Button>
			</div>
		</div>
	);
}
