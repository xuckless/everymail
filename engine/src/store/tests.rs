//! The query layer against a small replica in an in-memory database.

use super::{NewDraft, Store};
use crate::{
    backend::{NewMessage, Update},
    navigation::{Listing, Mailbox},
    preferences,
    proto::ui::v1::InboxCategory,
    world::{ApprovalStatus, DraftKind, Identity, Place, SendState, World},
};

/// Wednesday, November 11 2026, 9:50.
pub const NOW: i64 = 1_794_390_600;
pub const FIXTURE: &str = include_str!("fixture.sql");

/// The fixture, signed in.
pub fn fixture() -> Store {
    let store = Store::open(None).expect("an in-memory database");
    for update in [
        Update::SignedIn {
            user_name: "Kai".into(),
            provider: Identity::Google,
        },
        Update::Snapshot(FIXTURE.into()),
    ] {
        store.apply(&update, NOW).expect("the fixture");
    }
    store
}

fn ids(world: &World, listing: &Listing) -> Vec<String> {
    world
        .listing(listing)
        .expect("a known listing")
        .iter()
        .map(|t| t.id.clone())
        .collect()
}

fn for_you() -> Listing {
    Listing::Mailbox(Mailbox::All, InboxCategory::ForYou)
}

#[test]
fn listings_and_counts_follow_the_rows() {
    let s = fixture();
    let w = s.load(NOW, None).unwrap();
    assert_eq!(ids(&w, &for_you()), ["t-lake", "t-boat"], "newest first");
    assert_eq!(w.unread(None, Some(InboxCategory::ForYou)), 1);
    s.set_unread("t-lake", false).unwrap();
    let w = s.load(NOW, None).unwrap();
    assert_eq!(w.unread(None, Some(InboxCategory::ForYou)), 0);
    s.set_place("t-lake", Place::Archive).unwrap();
    let w = s.load(NOW, None).unwrap();
    assert_eq!(ids(&w, &for_you()), ["t-boat"]);
    assert_eq!(ids(&w, &Listing::Folder("archive".into())), ["t-lake"]);
}

#[test]
fn search_matches_prefixes_of_every_word_across_fields() {
    let s = fixture();
    assert_eq!(s.search("survey").unwrap(), ["t-lake"], "in the subject");
    assert_eq!(s.search("surv lak").unwrap(), ["t-lake"], "prefixes");
    assert_eq!(s.search("ravi").unwrap(), ["t-boat"], "the sender");
    assert_eq!(s.search("mo").unwrap(), ["t-boat"], "the recipients");
    assert!(
        s.search("\"").unwrap().is_empty(),
        "quotes are taken literally"
    );
    assert!(s.search("  ").unwrap().is_empty());
}

#[test]
fn mail_is_searchable_in_the_same_transaction_and_gone_with_its_account() {
    let s = fixture();
    s.apply(
        &Update::Mail {
            thread: None,
            thread_id: "t-lake".into(),
            message: NewMessage {
                id: "m-new".into(),
                from_you: false,
                sender: "Noor".into(),
                sent_at: NOW,
                body: "Bring a calculator to the zeppelin hangar".into(),
                attachments: vec![("map.pdf".into(), 1_000)],
            },
        },
        NOW,
    )
    .unwrap();
    assert_eq!(s.search("zeppelin").unwrap(), ["t-lake"]);
    let w = s.load(NOW, None).unwrap();
    let thread = w.thread("t-lake").unwrap();
    assert!(thread.unread, "new mail is unread");
    assert_eq!(thread.messages.last().unwrap().attachments.len(), 1);
    s.remove_account("work").unwrap();
    assert!(s.search("zeppelin").unwrap().is_empty());
    assert!(s.load(NOW, None).unwrap().thread("t-lake").is_none());
}

#[test]
fn a_send_keeps_the_state_the_backend_reports_then_lands_in_sent() {
    let s = fixture();
    let draft = s
        .create_draft(
            &NewDraft {
                kind: DraftKind::New,
                thread: None,
                message: None,
                approval: None,
                account: "mail",
                recipients: "Dana, Lee",
                subject: "Climbing on Saturday",
                body: "Who is in?",
            },
            NOW,
        )
        .unwrap();
    s.attach(&draft, "route.png", 2_048).unwrap();
    let send = |state: SendState| {
        s.apply(
            &Update::Outbox {
                draft: draft.clone(),
                state,
            },
            NOW,
        )
        .unwrap();
        s.load(NOW, None)
            .unwrap()
            .draft(&draft)
            .unwrap()
            .send
            .clone()
    };
    assert_eq!(
        send(SendState::Undoable(NOW + 10)),
        Some(SendState::Undoable(NOW + 10))
    );
    assert_eq!(send(SendState::Sending), Some(SendState::Sending));
    s.apply(&Update::SendCancelled(draft.clone()), NOW).unwrap();
    assert_eq!(s.load(NOW, None).unwrap().draft(&draft).unwrap().send, None);
    s.apply(
        &Update::Sent {
            draft: draft.clone(),
            message: "m-sent".into(),
            at: NOW + 11,
        },
        NOW + 11,
    )
    .unwrap();
    let w = s.load(NOW + 11, None).unwrap();
    assert!(w.draft(&draft).is_none());
    let sent = ids(&w, &Listing::Folder("sent".into()));
    assert_eq!(sent.first().map(String::as_str), Some("t-m-sent"));
    let thread = w.thread("t-m-sent").unwrap();
    assert_eq!(
        (thread.from.as_str(), thread.others.as_slice()),
        ("Dana", &["Lee".to_string()][..])
    );
    assert_eq!(thread.messages[0].attachments[0].name, "route.png");
    assert_eq!(s.search("climbing saturday").unwrap(), ["t-m-sent"]);
}

#[test]
fn a_snoozed_thread_comes_back_on_top_unread_when_the_snooze_ends() {
    let s = fixture();
    let snoozed = Update::Snoozed {
        thread: "t-boat".into(),
        until: NOW + 3_600,
    };
    s.apply(&snoozed, NOW).unwrap();
    let w = s.load(NOW, None).unwrap();
    assert_eq!(ids(&w, &for_you()), ["t-lake"]);
    s.apply(&Update::SnoozeEnded("t-boat".into()), NOW + 3_600)
        .unwrap();
    let w = s.load(NOW + 3_600, None).unwrap();
    assert_eq!(ids(&w, &for_you()), ["t-boat", "t-lake"]);
    assert!(w.thread("t-boat").unwrap().unread);
}

#[test]
fn a_released_mail_is_a_thread_in_the_inbox_and_its_sender_unblocked() {
    let s = fixture();
    s.release_quarantined("q-prize", "thread-x").unwrap();
    let w = s.load(NOW, None).unwrap();
    assert!(w.quarantine.is_empty());
    assert!(w.settings.blocked.is_empty());
    assert!(
        w.thread("thread-x")
            .is_some_and(|t| t.in_inbox(NOW) && t.unread)
    );
}

#[test]
fn approvals_keep_their_status_and_why_they_were_declined() {
    let s = fixture();
    s.resolving("a-noor").unwrap();
    let status = |s: &Store| {
        s.load(NOW, None)
            .unwrap()
            .approval("a-noor")
            .unwrap()
            .status
            .clone()
    };
    assert_eq!(status(&s), ApprovalStatus::Resolving);
    s.apply(
        &Update::Resolved {
            approval: "a-noor".into(),
            status: ApprovalStatus::Declined("Nobody answered".into()),
        },
        NOW,
    )
    .unwrap();
    assert_eq!(
        status(&s),
        ApprovalStatus::Declined("Nobody answered".into())
    );
}

#[test]
fn the_session_and_preferences_outlive_the_data() {
    let s = fixture();
    s.save_preferences(preferences::START).unwrap();
    s.remove_account("mail").unwrap();
    s.apply(&Update::Snapshot(FIXTURE.into()), NOW).unwrap();
    let w = s.load(NOW, None).unwrap();
    assert_eq!(w.accounts.len(), 2, "a snapshot replaces the data");
    assert_eq!(w.user.as_deref(), Some("Kai"));
    assert_eq!(
        s.search("survey").unwrap(),
        ["t-lake"],
        "the index is rebuilt"
    );
    s.reset().unwrap();
    let w = s.load(NOW, None).unwrap();
    assert!(
        w.accounts.is_empty() && w.user.is_some(),
        "reset keeps the session"
    );
    assert!(s.search("survey").unwrap().is_empty());
    s.sign_out().unwrap();
    assert!(s.load(NOW, None).unwrap().user.is_none());
    assert_eq!(s.preferences().unwrap(), Some(preferences::START));
}

#[test]
fn an_empty_database_migrates_to_the_one_schema_and_a_newer_one_is_refused() {
    let s = Store::open(None).unwrap();
    let version: i64 = s
        .connection()
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .unwrap();
    assert_eq!(version, super::MIGRATIONS.len() as i64);
    s.migrate()
        .expect("an up-to-date database migrates to itself");
    s.connection()
        .pragma_update(None, "user_version", version + 1)
        .unwrap();
    assert!(s.migrate().is_err(), "a newer schema is never used");
}

#[test]
fn an_unknown_code_or_missing_data_is_an_error() {
    let s = fixture();
    s.connection()
        .execute("UPDATE folders SET kind = 99 WHERE id = 'bills'", [])
        .unwrap();
    assert!(s.load(NOW, None).is_err(), "an unknown code");
    let s = fixture();
    s.connection()
        .execute("DELETE FROM cause_refs WHERE run_id = 'r-sorter'", [])
        .unwrap();
    assert!(
        s.load(NOW, None).is_err(),
        "a cause that needs a reference and has none"
    );
}

#[test]
fn references_go_with_their_owners() {
    let s = fixture();
    let count = |table: &str| -> i64 {
        s.connection()
            .query_row(&format!("SELECT count(*) FROM {table}"), [], |row| {
                row.get(0)
            })
            .unwrap()
    };
    s.clear_turns("helper").unwrap();
    assert_eq!(
        count("turn_refs"),
        0,
        "a cleared chat takes its answers' refs"
    );
    s.remove_agent("sorter").unwrap();
    assert_eq!((count("cause_refs"), count("step_refs")), (0, 0));
}

#[test]
fn an_attachment_is_removed_by_its_id_and_the_others_keep_theirs() {
    let s = fixture();
    let draft = s
        .create_draft(
            &NewDraft {
                kind: DraftKind::New,
                thread: None,
                message: None,
                approval: None,
                account: "mail",
                recipients: "",
                subject: "",
                body: "",
            },
            NOW,
        )
        .unwrap();
    for name in ["a.pdf", "b.png", "c.zip"] {
        s.attach(&draft, name, 1).unwrap();
    }
    let attachments = |s: &Store| {
        s.load(NOW, None)
            .unwrap()
            .draft(&draft)
            .unwrap()
            .attachments
            .clone()
    };
    let before = attachments(&s);
    s.remove_attachment(&draft, &before[1].0).unwrap();
    let after = attachments(&s);
    assert_eq!(after, vec![before[0].clone(), before[2].clone()]);
    s.attach(&draft, "d.txt", 1).unwrap();
    let again = attachments(&s);
    assert_eq!(again.len(), 3);
    assert_eq!(again[2].1, "d.txt", "a new attachment goes last");
    assert!(
        !before.iter().any(|(id, ..)| *id == again[2].0),
        "under an id never used before"
    );
}
