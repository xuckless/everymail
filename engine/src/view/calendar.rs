use crate::{
    demo::{CalendarEvent, World},
    navigation::{Catalog, Navigation, Selection},
    proto::ui::v1::{
        self as ui, calendar_day::Agenda, calendar_event::Kind, calendar_view::Selected,
        invitation_answer::State as Answer,
    },
};

/// "9:00", "19:30"
fn time(minute: u32) -> String {
    format!("{}:{:02}", minute / 60, minute % 60)
}

fn source<'a>(world: &'a World, event: &CalendarEvent) -> Option<&'a ui::CalendarSource> {
    world
        .calendar
        .sources
        .iter()
        .find(|s| s.id == event.calendar)
}

/// The phone agenda's line: "10:00–11:30 · FA-110 · Laurentian", "Mira invited you · tap to reply"
fn agenda_line(event: &CalendarEvent, world: &World) -> String {
    let calendar = source(world, event).map_or("", |s| s.name.as_str());
    let span = format!("{}–{}", time(event.start), time(event.end));
    let place = if event.place.is_empty() {
        String::new()
    } else {
        format!(" · {}", event.place)
    };
    let answered = |answer: &str| format!("{answer} · {span} · {calendar}");
    match answer(event) {
        Some(Answer::Unanswered(_)) => format!(
            "{} · tap to reply",
            event
                .invitation
                .as_ref()
                .map_or("", |i| i.invited_by.as_str())
        ),
        Some(Answer::Accepted(_)) => answered("You accepted"),
        Some(Answer::Tentative(_)) => answered("Maybe"),
        Some(Answer::Declined(_)) => answered("Declined"),
        None => format!("{span}{place} · {calendar}"),
    }
}

fn answer(event: &CalendarEvent) -> Option<&Answer> {
    event
        .invitation
        .as_ref()
        .and_then(|i| i.answer.as_ref())
        .and_then(|a| a.state.as_ref())
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
        Some(Answer::Unanswered(_)) => "Invitation · reply".into(),
        Some(Answer::Tentative(_)) => format!("Maybe · {start}"),
        Some(Answer::Declined(_)) => "Declined".into(),
        Some(Answer::Accepted(_)) | None => when,
    }
}

pub fn calendar(navigation: &Navigation, world: &World) -> ui::CalendarView {
    let selection = &navigation.event;
    let calendar = &world.calendar;
    let shown = world.events();
    let events: Vec<&CalendarEvent> = calendar
        .events
        .iter()
        .filter(|e| shown.contains(&e.id))
        .collect();
    let chosen = match selection {
        Selection::Id(id) => events.iter().find(|e| e.id == id),
        Selection::None => None,
    };
    let selected = match chosen {
        None => Selected::None(ui::NoEventSelected {}),
        Some(event) => match &event.invitation {
            Some(invitation) => Selected::Invitation(invitation.clone()),
            None => Selected::Event(ui::EventDetail {
                id: event.id.into(),
                heading: "Edit event".into(),
                title: event.title.into(),
                days: calendar
                    .days
                    .iter()
                    .enumerate()
                    .map(|(i, d)| ui::Choice {
                        id: d.id.into(),
                        label: format!("{}, {} {}", d.weekday, d.month, d.day),
                        selected: i == event.day,
                    })
                    .collect(),
                starts: time(event.start),
                ends: time(event.end),
                calendars: calendar
                    .sources
                    .iter()
                    .map(|s| ui::CalendarChoice {
                        id: s.id.clone(),
                        name: s.name.clone(),
                        color: s.color,
                        selected: s.id == event.calendar,
                    })
                    .collect(),
                saved_to: source(world, event)
                    .map_or(String::new(), |s| format!("Saved to {}", s.provider_name)),
                guests: event.guests.iter().map(|g| (*g).into()).collect(),
            }),
        },
    };
    ui::CalendarView {
        month: calendar.month.into(),
        range: calendar.range.into(),
        hours: (calendar.first_hour..=calendar.last_hour)
            .map(|hour| ui::HourMark {
                minute: hour * 60,
                label: time(hour * 60),
            })
            .collect(),
        days: calendar
            .days
            .iter()
            .enumerate()
            .map(|(i, d)| {
                let on_day: Vec<ui::CalendarEvent> = events
                    .iter()
                    .filter(|e| e.day == i)
                    .map(|e| ui::CalendarEvent {
                        id: e.id.into(),
                        color: source(world, e).map_or(0, |s| s.color),
                        start: time(e.start),
                        agenda_line: agenda_line(e, world),
                        title: e.title.into(),
                        detail: detail(e),
                        start_minute: e.start,
                        end_minute: e.end,
                        selected: chosen.is_some_and(|c| c.id == e.id),
                        kind: Some(match &e.invitation {
                            Some(invitation) => {
                                Kind::Invitation(invitation.answer.unwrap_or_default())
                            }
                            None => Kind::Plain(ui::PlainEvent {}),
                        }),
                    })
                    .collect();
                ui::CalendarDay {
                    weekday: d.weekday.into(),
                    day: d.day.into(),
                    today: d.today,
                    id: d.id.into(),
                    selected: matches!(&navigation.day, Selection::Id(day) if day == d.id),
                    title: format!(
                        "{}{} {} {}",
                        if d.today { "Today, " } else { "" },
                        d.weekday,
                        d.month,
                        d.day
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
        calendars: calendar.sources.clone(),
        invitations: events.iter().filter_map(|e| e.invitation.clone()).collect(),
        selected: Some(selected),
    }
}
