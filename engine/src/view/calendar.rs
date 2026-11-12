use super::{mail::invitation, time_choices};
use crate::{
    date::{Date, WEEKDAYS},
    navigation::{Catalog, Drawer, Mode, Navigation},
    proto::ui::v1::{
        self as ui,
        calendar_day::Agenda,
        calendar_event::Kind,
        calendar_view::{Period, Selected},
    },
    time::{self, clock as time},
    world::{Answer, CalendarEvent, CalendarSource, World},
};

/// The hours the week grid shows.
const FIRST_HOUR: u32 = 8;
const LAST_HOUR: u32 = 20;

fn source<'a>(world: &'a World, event: &CalendarEvent) -> Option<&'a CalendarSource> {
    world.calendar(&event.calendar)
}

fn color(world: &World, event: &CalendarEvent) -> i32 {
    source(world, event).map_or(0, |s| s.color.into())
}

/// The time of the next event today that has not ended, for the sidebar: "10:00"
pub fn next_event(world: &World) -> String {
    let today = world.today();
    let minute = time::minute(world.now);
    let shown = world.events();
    world
        .events
        .iter()
        .filter(|e| e.date == today && e.end > minute && shown.contains(&e.id.as_str()))
        .filter(|e| {
            e.invitation
                .as_ref()
                .is_none_or(|i| i.answer != Answer::Declined)
        })
        .map(|e| e.start)
        .min()
        .map_or(String::new(), time)
}

fn answer(event: &CalendarEvent) -> Option<Answer> {
    event.invitation.as_ref().map(|i| i.answer)
}

/// The phone agenda's line: "10:00–11:30 · FA-110 · Laurentian", "Mira invited you · tap to reply"
fn agenda_line(event: &CalendarEvent, world: &World) -> String {
    let calendar = source(world, event).map_or("", |s| s.name.as_str());
    let invited_by = event
        .invitation
        .as_ref()
        .map_or("", |i| i.invited_by.as_str());
    let span = format!("{}–{}", time(event.start), time(event.end));
    let place = if event.place.is_empty() {
        String::new()
    } else {
        format!(" · {}", event.place)
    };
    let answered = |answer: &str| format!("{answer} · {span} · {calendar}");
    match answer(event) {
        Some(Answer::Unanswered) => format!("{invited_by} invited you · tap to reply"),
        Some(Answer::Accepted) => answered("You accepted"),
        Some(Answer::Tentative) => answered("Maybe"),
        Some(Answer::Declined) => answered("Declined"),
        None => format!("{span}{place} · {calendar}"),
    }
}

/// The line under an event's title: "10:00 · FA-110", "Invitation · reply"
fn detail(event: &CalendarEvent) -> String {
    let start = time(event.start);
    let when = if event.place.is_empty() {
        start.clone()
    } else {
        format!("{start} · {}", event.place)
    };
    match answer(event) {
        Some(Answer::Unanswered) => "Invitation · reply".into(),
        Some(Answer::Tentative) => format!("Maybe · {start}"),
        Some(Answer::Declined) => "Declined".into(),
        Some(Answer::Accepted) | None => when,
    }
}

const MODES: [(Mode, &str); 3] = [
    (Mode::Week, "Week"),
    (Mode::Month, "Month"),
    (Mode::Year, "Year"),
];

/// "Mon, Nov 9"
fn day_label(day: Date) -> String {
    format!(
        "{}, {} {}",
        WEEKDAYS[day.weekday()],
        day.month_short(),
        day.day
    )
}

/// "Nov 9 – 15", "Nov 30 – Dec 6", "Dec 28, 2026 – Jan 3, 2027"
fn range(first: Date, last: Date) -> String {
    if first.year != last.year {
        format!(
            "{} {}, {} – {} {}, {}",
            first.month_short(),
            first.day,
            first.year,
            last.month_short(),
            last.day,
            last.year
        )
    } else if first.month != last.month {
        format!(
            "{} {} – {} {}",
            first.month_short(),
            first.day,
            last.month_short(),
            last.day
        )
    } else {
        format!("{} {} – {}", first.month_short(), first.day, last.day)
    }
}

/// The drawer with the event's values.
fn drawer(event: &CalendarEvent, heading: &str, world: &World) -> ui::EventDetail {
    ui::EventDetail {
        id: event.id.clone(),
        heading: heading.into(),
        title: event.title.clone(),
        day_id: event.date.id(),
        start_minute: event.start,
        end_minute: event.end,
        times: time_choices(),
        days: (0..7)
            .map(|i| event.date.week_start().add_days(i))
            .map(|d| ui::Choice {
                id: d.id(),
                label: day_label(d),
                selected: d == event.date,
            })
            .collect(),
        starts: time(event.start),
        ends: time(event.end),
        calendars: world
            .calendars
            .iter()
            .map(|s| ui::CalendarChoice {
                id: s.id.clone(),
                name: s.name.clone(),
                color: s.color.into(),
                selected: s.id == event.calendar,
            })
            .collect(),
        saved_to: source(world, event)
            .map_or(String::new(), |s| format!("Saved to {}", s.provider_name)),
        guests: event.guests.clone(),
        place: event.place.clone(),
    }
}

/// A new event's values before it is saved: 9:00 to 10:00 on the selected day, on the first
/// calendar shown.
fn unsaved(day: Date, world: &World) -> CalendarEvent {
    let calendar = world
        .calendars
        .iter()
        .find(|c| c.shown)
        .or(world.calendars.first());
    CalendarEvent {
        id: String::new(),
        calendar: calendar.map_or(String::new(), |c| c.id.clone()),
        date: day,
        start: 9 * 60,
        end: 10 * 60,
        title: String::new(),
        place: String::new(),
        guests: Vec::new(),
        invitation: None,
    }
}

fn plain_event(
    e: &CalendarEvent,
    chosen: Option<&&CalendarEvent>,
    navigation: &Navigation,
    world: &World,
) -> ui::CalendarEvent {
    ui::CalendarEvent {
        id: e.id.clone(),
        color: color(world, e),
        start: time(e.start),
        agenda_line: agenda_line(e, world),
        title: e.title.clone(),
        detail: detail(e),
        start_minute: e.start,
        end_minute: e.end,
        selected: chosen.is_some_and(|c| c.id == e.id),
        kind: Some(
            match invitation(e, &navigation.open, world).and_then(|i| i.answer) {
                Some(answer) => Kind::Invitation(answer),
                None => Kind::Plain(ui::PlainEvent {}),
            },
        ),
    }
}

pub fn calendar(navigation: &Navigation, world: &World) -> ui::CalendarView {
    let today = world.today();
    let selected_day = navigation.day;
    let shown = world.events();
    let mut events: Vec<&CalendarEvent> = world
        .events
        .iter()
        .filter(|e| shown.contains(&e.id.as_str()))
        .collect();
    events.sort_by_key(|e| (e.date, e.start));
    let on = |day: Date| events.iter().copied().filter(move |e| e.date == day);
    let chosen = match &navigation.event {
        Drawer::Event(id) => events.iter().find(|e| e.id == *id),
        Drawer::Closed | Drawer::New => None,
    };
    let selected = match (&navigation.event, chosen) {
        (Drawer::New, _) => {
            Selected::Unsaved(drawer(&unsaved(selected_day, world), "New event", world))
        }
        (Drawer::Closed | Drawer::Event(_), None) => Selected::None(ui::NoEventSelected {}),
        (Drawer::Closed | Drawer::Event(_), Some(event)) => {
            match invitation(event, &navigation.open, world) {
                Some(invitation) => Selected::Invitation(ui::SelectedInvitation {
                    invitation: Some(invitation),
                    day_id: event.date.id(),
                }),
                None => Selected::Event(drawer(event, "Edit event", world)),
            }
        }
    };
    let month_title = format!("{} {}", selected_day.month_name(), selected_day.year);
    let (title, period) = match navigation.calendar {
        Mode::Month => (
            month_title,
            Period::Month(ui::MonthPeriod {
                weekdays: WEEKDAYS.iter().map(|d| (*d).into()).collect(),
                weeks: selected_day
                    .month_weeks()
                    .iter()
                    .map(|week| ui::MonthWeek {
                        days: week
                            .iter()
                            .map(|&day| {
                                let all: Vec<&CalendarEvent> = on(day).collect();
                                let shown = if all.len() > 3 { 2 } else { all.len() };
                                ui::MonthDay {
                                    id: day.id(),
                                    day: day.day.to_string(),
                                    in_month: day.month == selected_day.month,
                                    today: day == today,
                                    selected: day == selected_day,
                                    chips: all[..shown]
                                        .iter()
                                        .map(|e| ui::EventChip {
                                            id: e.id.clone(),
                                            title: e.title.clone(),
                                            color: color(world, e),
                                            start: time(e.start),
                                        })
                                        .collect(),
                                    more: if all.len() > shown {
                                        format!("+{} more", all.len() - shown)
                                    } else {
                                        String::new()
                                    },
                                }
                            })
                            .collect(),
                    })
                    .collect(),
            }),
        ),
        Mode::Year => (
            selected_day.year.to_string(),
            Period::Year(ui::YearPeriod {
                weekdays: WEEKDAYS.iter().map(|d| d[..1].into()).collect(),
                months: (1..=12)
                    .filter_map(|month| Date::new(selected_day.year, month, 1))
                    .map(|first| ui::YearMonth {
                        id: first.month_id(),
                        name: first.month_name().into(),
                        current: first.month == selected_day.month,
                        weeks: first
                            .month_weeks()
                            .iter()
                            .map(|week| ui::YearWeek {
                                days: week
                                    .iter()
                                    .map(|&day| ui::YearDay {
                                        id: day.id(),
                                        day: day.day.to_string(),
                                        in_month: day.month == first.month,
                                        today: day == today,
                                        busy: on(day).next().is_some(),
                                    })
                                    .collect(),
                            })
                            .collect(),
                    })
                    .collect(),
            }),
        ),
        Mode::Week => {
            let monday = selected_day.week_start();
            (
                month_title,
                Period::Week(ui::WeekPeriod {
                    range: range(monday, monday.add_days(6)),
                    hours: (FIRST_HOUR..=LAST_HOUR)
                        .map(|hour| ui::HourMark {
                            minute: hour * 60,
                            label: time(hour * 60),
                        })
                        .collect(),
                    days: (0..7)
                        .map(|i| monday.add_days(i))
                        .map(|day| {
                            let on_day: Vec<ui::CalendarEvent> = on(day)
                                .map(|e| plain_event(e, chosen, navigation, world))
                                .collect();
                            let weekday = WEEKDAYS[day.weekday()];
                            ui::CalendarDay {
                                weekday: weekday.into(),
                                day: day.day.to_string(),
                                today: day == today,
                                id: day.id(),
                                selected: day == selected_day,
                                title: format!(
                                    "{}{weekday} {} {}",
                                    if day == today { "Today, " } else { "" },
                                    day.month_short(),
                                    day.day
                                ),
                                agenda: Some(if on_day.is_empty() {
                                    Agenda::Nothing(ui::NothingPlanned {
                                        text: "Nothing planned".into(),
                                    })
                                } else {
                                    Agenda::Events(ui::DayEvents { events: on_day })
                                }),
                            }
                        })
                        .collect(),
                }),
            )
        }
    };
    ui::CalendarView {
        title,
        modes: MODES
            .iter()
            .map(|(mode, label)| ui::CalendarModeTab {
                mode: ui::CalendarMode::from(*mode).into(),
                label: (*label).into(),
                current: *mode == navigation.calendar,
            })
            .collect(),
        period: Some(period),
        calendars: world
            .calendars
            .iter()
            .map(|s| ui::CalendarSource {
                id: s.id.clone(),
                name: s.name.clone(),
                provider: s.provider.into(),
                provider_name: s.provider_name.clone(),
                shown: s.shown,
                color: s.color.into(),
            })
            .collect(),
        invitations: events
            .iter()
            .filter_map(|e| invitation(e, &navigation.open, world))
            .collect(),
        selected: Some(selected),
    }
}
