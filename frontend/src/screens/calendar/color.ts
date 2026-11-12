import { unset } from "../../engine";
import { CalendarColor } from "../../gen/ui/v1/calendar_pb";

// The `data-cal` a calendar's colour is drawn with (calendar.css).
export function calendarColor(color: CalendarColor) {
	switch (color) {
		case CalendarColor.VIOLET:
			return "violet";
		case CalendarColor.CYAN:
			return "cyan";
		case CalendarColor.AMBER:
			return "amber";
		case CalendarColor.UNSPECIFIED:
			return unset("CalendarColor");
	}
}
