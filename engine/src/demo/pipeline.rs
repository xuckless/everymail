//! New mail on its way in, as the Backend takes it: the user's rules, the block list, then
//! Gatekeeper's sorting, then the events that wake other agents. Also the mail the sample world
//! keeps receiving as time goes on.

use super::{
    matcher::{Event, Fired},
    policy::quarantine_ends,
};
use crate::{
    backend::{NewMessage, NewQuarantined, NewThread, Update},
    proto::ui::v1::{InboxCategory, NoteKind},
    time,
    world::{AccountState, Condition, Place, RuleAction, World, category_id, category_name},
};

/// What the provider's headers and verdicts say a mail is. Gatekeeper's code stage reads them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Person,
    /// Bulk mail: receipts, notices, deliveries.
    Bulk,
    Newsletter,
    Scam,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Incoming {
    pub thread: String,
    pub account: String,
    pub sender: String,
    pub address: String,
    pub subject: String,
    pub body: String,
    pub kind: Kind,
    pub at: i64,
}

/// What a rule is checked against.
pub struct Mail<'a> {
    pub sender: &'a str,
    pub address: &'a str,
    pub subject: &'a str,
    pub newsletter: bool,
    pub link: bool,
    /// The user has written to the sender.
    pub known: bool,
}

/// Whether a rule's condition holds for a mail. Stored text is in lower case.
pub fn holds(condition: &Condition, mail: &Mail) -> bool {
    let address = mail.address.to_lowercase();
    match condition {
        Condition::SenderContains(who) => {
            address.contains(who.as_str()) || mail.sender.to_lowercase().contains(who.as_str())
        }
        Condition::SenderDomain(domain) => address
            .rsplit_once('@')
            .is_some_and(|(_, d)| d == domain || d.ends_with(&format!(".{domain}"))),
        Condition::SubjectContains(words) => {
            let subject = mail.subject.to_lowercase();
            words.iter().any(|w| subject.contains(w.as_str()))
        }
        Condition::Newsletter => mail.newsletter,
        Condition::NewSenderWithLink => !mail.known && mail.link,
    }
}

/// What became of a mail.
pub struct Arrival {
    pub update: Update,
    /// Gatekeeper sorts it: it runs on the mail, and `then` fires when it has finished.
    pub gatekeeper: bool,
    pub then: Vec<Fired>,
}

/// Rules first, then the block list, then Gatekeeper. A `sweep` is mail already in an account when
/// it is connected: sorted, but it wakes nobody.
pub fn arrive(mail: &Incoming, world: &World, sweep: bool) -> Arrival {
    let facts = Mail {
        sender: &mail.sender,
        address: &mail.address,
        subject: &mail.subject,
        newsletter: mail.kind == Kind::Newsletter,
        link: mail.body.contains("http"),
        known: world.known(&mail.sender),
    };
    let actions: Vec<&RuleAction> = world
        .settings
        .rules
        .iter()
        .filter(|r| r.enabled && holds(&r.condition, &facts))
        .flat_map(|r| &r.actions)
        .collect();
    let blocked = world
        .settings
        .blocked
        .iter()
        .any(|b| b.address.eq_ignore_ascii_case(&mail.address));
    if blocked {
        return Arrival {
            update: Update::Quarantined(NewQuarantined {
                id: format!("q-{}", mail.thread),
                account: mail.account.clone(),
                sender: mail.address.clone(),
                received_at: mail.at,
                subject: mail.subject.clone(),
                body: mail.body.clone(),
                reason: "Blocked sender".into(),
                caught_by: "Your block list".into(),
                why: "This sender is on your block list, so no agent read it".into(),
                outcome: "Kept out of your inbox".into(),
                thread: None,
                block: None,
                drops_at: quarantine_ends(mail.at, world),
            }),
            gatekeeper: false,
            then: Vec::new(),
        };
    }
    let gatekeeper = world.agent("gatekeeper").is_some_and(|a| a.active);
    let mut category = match (gatekeeper, mail.kind) {
        (true, Kind::Bulk) => InboxCategory::Updates,
        (true, Kind::Newsletter) => InboxCategory::Reading,
        (true, Kind::Person | Kind::Scam) | (false, _) => InboxCategory::ForYou,
    };
    let mut place = Place::Inbox;
    let mut folder = None;
    let mut note: Option<(String, NoteKind)> = None;
    let mut then = Vec::new();
    let event = |event: Event| Fired::outside(event);
    for action in actions {
        match action {
            RuleAction::MoveTo(id) => {
                let name = world
                    .folders
                    .iter()
                    .find(|f| f.id == *id)
                    .map_or("", |f| f.name.as_str());
                folder = Some(id.clone());
                place = Place::Archive;
                note = Some((format!("Your rule filed this in {name}"), NoteKind::Filed));
                then.push(event(Event::Sorted {
                    thread: mail.thread.clone(),
                    into: id.clone(),
                }));
            }
            RuleAction::Archive => {
                place = Place::Archive;
                note = Some((
                    "Your rule kept this out of the inbox".into(),
                    NoteKind::Filed,
                ));
            }
            RuleAction::Trash => place = Place::Trash,
            RuleAction::MarkImportant => {
                note = Some(("Your rule marked this important".into(), NoteKind::Flagged));
                then.push(event(Event::Flagged(mail.thread.clone())));
            }
            RuleAction::HandTo(agent) => then.push(event(Event::Handoff {
                to: agent.clone(),
                by: "your rule".into(),
                thread: mail.thread.clone(),
            })),
            RuleAction::KeepForBriefing => {
                category = InboxCategory::Reading;
                place = Place::Archive;
                note = Some((
                    "Your rule saved this for the briefing".into(),
                    NoteKind::Filed,
                ));
            }
        }
    }
    if gatekeeper && mail.kind == Kind::Scam {
        note = Some((
            "Gatekeeper flagged this as a likely scam".into(),
            NoteKind::Flagged,
        ));
        then = vec![event(Event::Handoff {
            to: "security-guard".into(),
            by: "Gatekeeper".into(),
            thread: mail.thread.clone(),
        })];
    } else {
        if gatekeeper {
            let into = category_name(category);
            if note.is_none() {
                note = Some(if mail.kind == Kind::Person && facts.known {
                    then.push(event(Event::Flagged(mail.thread.clone())));
                    ("Gatekeeper flagged this for you".into(), NoteKind::Flagged)
                } else {
                    (
                        format!("Gatekeeper sorted this into {into}"),
                        NoteKind::Filed,
                    )
                });
            }
            then.push(event(Event::Sorted {
                thread: mail.thread.clone(),
                into: category_id(category).into(),
            }));
        }
        then.push(event(Event::NewMail(mail.thread.clone())));
    }
    Arrival {
        update: Update::Mail {
            thread: Some(NewThread {
                id: mail.thread.clone(),
                account: mail.account.clone(),
                place,
                category,
                from: mail.sender.clone(),
                subject: mail.subject.clone(),
                others: Vec::new(),
                note,
                folder,
            }),
            thread_id: mail.thread.clone(),
            message: NewMessage {
                id: format!("{}-m", mail.thread),
                from_you: false,
                sender: mail.sender.clone(),
                sent_at: mail.at,
                body: mail.body.clone(),
                attachments: Vec::new(),
            },
        },
        gatekeeper,
        then: if sweep { Vec::new() } else { then },
    }
}

/// A mail the sample world receives, this many minutes after the sample's now.
struct Delivery {
    minutes: i64,
    account: &'static str,
    sender: &'static str,
    address: &'static str,
    subject: &'static str,
    body: &'static str,
    kind: Kind,
}

const POOL: [Delivery; 8] = [
    Delivery {
        minutes: 2,
        account: "laurentian",
        sender: "Prof. Lindqvist",
        address: "lindqvist@laurentian.ca",
        subject: "COSC 4P80 project deadline moved to Dec 4",
        body: "Hello all,\n\nThe project deadline moves from December 1 to December 4 at 23:59. The rubric is unchanged.\n\nBest,\nProf. Lindqvist",
        kind: Kind::Person,
    },
    Delivery {
        minutes: 4,
        account: "personal",
        sender: "Ridgeway Hosting",
        address: "billing@ridgewayhosting.com",
        subject: "Receipt for your November invoice",
        body: "Thanks for your payment of $12.00 for invoice #4519.\n\nRidgeway Hosting billing",
        kind: Kind::Bulk,
    },
    Delivery {
        minutes: 6,
        account: "personal",
        sender: "The Weekly Digest",
        address: "issue@weeklydigest.news",
        subject: "This week: what changed in distributed systems",
        body: "Five stories worth your time this week, starting with consensus without leaders.\n\nRead online: https://weeklydigest.news/issue/212",
        kind: Kind::Newsletter,
    },
    Delivery {
        minutes: 8,
        account: "personal",
        sender: "Account Security",
        address: "security@acc0unt-verify.net",
        subject: "Your account will be closed today",
        body: "Confirm your password within 24 hours: https://acc0unt-verify.net/confirm",
        kind: Kind::Scam,
    },
    Delivery {
        minutes: 10,
        account: "personal",
        sender: "claims@prize-center-win.top",
        address: "claims@prize-center-win.top",
        subject: "You have won a laptop",
        body: "Claim your prize now: https://prize-center-win.top/claim",
        kind: Kind::Scam,
    },
    Delivery {
        minutes: 12,
        account: "laurentian",
        sender: "Mira",
        address: "mira@laurentian.ca",
        subject: "Section 3 notes for tonight",
        body: "Hi Ali,\n\nI put my notes for section 3 in our folder. Can you look before you start the slides?\n\nMira",
        kind: Kind::Person,
    },
    Delivery {
        minutes: 14,
        account: "personal",
        sender: "Parcelpoint",
        address: "track@parcelpoint.com",
        subject: "Your parcel is out for delivery",
        body: "Your parcel from Northwind Books arrives today between 13:00 and 17:00.",
        kind: Kind::Bulk,
    },
    Delivery {
        minutes: 16,
        account: "icloud",
        sender: "Cloudberry Photos",
        address: "albums@cloudberry.photos",
        subject: "12 new photos in Climbing trip",
        body: "Jordan added 12 photos to the shared album Climbing trip.",
        kind: Kind::Bulk,
    },
];

fn incoming(i: usize, d: &Delivery, at: i64) -> Incoming {
    Incoming {
        thread: format!("demo-mail-{i}"),
        account: d.account.into(),
        sender: d.sender.into(),
        address: d.address.into(),
        subject: d.subject.into(),
        body: d.body.into(),
        kind: d.kind,
        at,
    }
}

/// When each of the sample's mails arrives, for the accounts that are signed in: after the account
/// was last checked.
fn deliveries(world: &World, base: i64) -> impl Iterator<Item = (usize, &Delivery, i64)> {
    POOL.iter().enumerate().filter_map(move |(i, d)| {
        let at = base + d.minutes * time::MINUTE;
        world
            .account(d.account)
            .filter(|a| a.state == AccountState::Synced && a.synced_at < at)
            .map(|_| (i, d, at))
    })
}

/// The sample's mail that has arrived by now and is not in the replica yet.
pub fn due(world: &World, base: i64) -> Vec<Incoming> {
    deliveries(world, base)
        .filter(|(_, _, at)| *at <= world.now)
        .map(|(i, d, at)| incoming(i, d, at))
        .collect()
}

pub fn next(world: &World, base: i64) -> Option<i64> {
    deliveries(world, base)
        .map(|(_, _, at)| at)
        .filter(|at| *at > world.now)
        .min()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        demo::tests::{SAMPLE_NOW, world},
        world::Place,
    };

    fn base() -> i64 {
        SAMPLE_NOW
    }

    fn thread(arrival: &Arrival) -> &NewThread {
        match &arrival.update {
            Update::Mail {
                thread: Some(t), ..
            } => t,
            _ => panic!("not a new thread"),
        }
    }

    #[test]
    fn rules_apply_before_gatekeeper_and_hand_mail_over() {
        let (_, mut w) = world();
        w.now = base() + 20 * time::MINUTE;
        let mail = due(&w, base());
        assert_eq!(mail.len(), 7, "the iCloud account is signed out");
        // The laurentian.ca rule marks it important; Gatekeeper sorts it into For you.
        let lindqvist = arrive(&mail[0], &w, false);
        assert!(lindqvist.gatekeeper);
        assert_eq!(
            thread(&lindqvist).note,
            Some(("Your rule marked this important".into(), NoteKind::Flagged))
        );
        assert_eq!(
            lindqvist.then.iter().map(|f| &f.event).collect::<Vec<_>>(),
            vec![
                &Event::Flagged("demo-mail-0".into()),
                &Event::Sorted {
                    thread: "demo-mail-0".into(),
                    into: "for-you".into()
                },
                &Event::NewMail("demo-mail-0".into()),
            ]
        );
        // The receipt rule hands it to Receipts clerk; Gatekeeper sorts it into Updates.
        let receipt = arrive(&mail[1], &w, false);
        assert_eq!(thread(&receipt).category, InboxCategory::Updates);
        assert_eq!(
            receipt.then[0].event,
            Event::Handoff {
                to: "receipts-clerk".into(),
                by: "your rule".into(),
                thread: "demo-mail-1".into()
            }
        );
        // The newsletter rule keeps it out of the inbox.
        let digest = arrive(&mail[2], &w, false);
        assert_eq!(thread(&digest).place, Place::Archive);
        // A blocked sender goes to Quarantine and wakes nobody.
        let blocked = arrive(&mail[3], &w, false);
        assert!(matches!(blocked.update, Update::Quarantined(_)) && blocked.then.is_empty());
        // A scam goes to Security Guard only.
        let scam = arrive(&mail[4], &w, false);
        assert_eq!(scam.then.len(), 1);
        // A sweep wakes nobody.
        assert!(arrive(&mail[0], &w, true).then.is_empty());
    }

    #[test]
    fn a_rule_the_user_adds_applies_to_later_mail() {
        let (store, _) = world();
        store
            .add_rule(
                "rule-x",
                &Condition::SenderContains("track@parcelpoint.com".into()),
                &[RuleAction::MoveTo("receipts".into())],
            )
            .unwrap();
        let w = store.load(base() + 20 * time::MINUTE, None).unwrap();
        let parcel = due(&w, base())
            .into_iter()
            .find(|m| m.sender == "Parcelpoint");
        let arrival = arrive(&parcel.unwrap(), &w, false);
        assert_eq!(thread(&arrival).folder.as_deref(), Some("receipts"));
        assert!(arrival.then.iter().any(|f| f.event
            == Event::Sorted {
                thread: "demo-mail-6".into(),
                into: "receipts".into()
            }));
    }

    fn mail<'a>(sender: &'a str, address: &'a str, subject: &'a str) -> Mail<'a> {
        Mail {
            sender,
            address,
            subject,
            newsletter: false,
            link: false,
            known: false,
        }
    }

    #[test]
    fn conditions_hold_on_the_mail_they_describe() {
        let domain = Condition::SenderDomain("laurentian.ca".into());
        assert!(holds(&domain, &mail("Prof", "lindqvist@laurentian.ca", "")));
        assert!(holds(&domain, &mail("Prof", "x@cs.laurentian.ca", "")));
        assert!(!holds(&domain, &mail("Prof", "x@notlaurentian.ca", "")));
        let subject = Condition::SubjectContains(vec!["receipt".into(), "invoice".into()]);
        assert!(holds(
            &subject,
            &mail("Ridgeway", "b@r.com", "Your November Invoice")
        ));
        assert!(!holds(&subject, &mail("Ridgeway", "b@r.com", "Hello")));
        let sender = Condition::SenderContains("mira".into());
        assert!(holds(&sender, &mail("Mira Castellanos", "m@x.ca", "")));
        let mut m = mail("a", "a@b.c", "");
        m.link = true;
        assert!(holds(&Condition::NewSenderWithLink, &m));
        m.known = true;
        assert!(!holds(&Condition::NewSenderWithLink, &m));
    }

    #[test]
    fn a_disabled_rule_does_nothing() {
        let (store, _) = world();
        store.set_rule_enabled("rule-laurentian", false).unwrap();
        let w = store.load(base() + 20 * time::MINUTE, None).unwrap();
        let lindqvist = arrive(&due(&w, base())[0], &w, false);
        assert_eq!(
            thread(&lindqvist).note,
            Some((
                "Gatekeeper sorted this into For you".into(),
                NoteKind::Filed
            ))
        );
    }

    #[test]
    fn without_gatekeeper_mail_lands_in_for_you_and_wakes_at_once() {
        let (store, _) = world();
        store.set_agent_active("gatekeeper", false).unwrap();
        let w = store.load(base() + 20 * time::MINUTE, None).unwrap();
        let mail = due(&w, base());
        let parcel = arrive(&mail[6], &w, false);
        assert!(!parcel.gatekeeper);
        assert_eq!(thread(&parcel).category, InboxCategory::ForYou);
        assert_eq!(
            parcel.then.iter().map(|f| &f.event).collect::<Vec<_>>(),
            vec![&Event::NewMail("demo-mail-6".into())]
        );
    }

    #[test]
    fn mail_arrives_once_after_the_account_was_checked() {
        let (_, mut w) = world();
        assert_eq!(next(&w, base()), Some(base() + 2 * time::MINUTE));
        w.now = base() + 5 * time::MINUTE;
        assert_eq!(due(&w, base()).len(), 2);
        for a in &mut w.accounts {
            a.synced_at = w.now;
        }
        assert!(due(&w, base()).is_empty());
        assert_eq!(next(&w, base()), Some(base() + 6 * time::MINUTE));
    }
}
