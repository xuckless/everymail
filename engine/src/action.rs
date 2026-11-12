//! What the user's intents do to the data: each action is checked, applied to the replica in one
//! transaction, and may ask the Backend for work, move the navigation, and leave an outcome or a
//! notice. The checks and the date arithmetic are pure functions.

use crate::{
    backend::{Request, SendAt, Work},
    builder,
    date::Date,
    live::{self, DryRun, Export, Field, Generation, ImapCheck, Live, SignIn, Target, Undo},
    navigation::{self, ConnectStep, Disclosure, Navigation, PaneChoice, Route, Screen, Selection},
    proto::ui::v1::{Form, MailProvider, ModelProvider, PermissionLevel},
    schedule::Schedule,
    store::{self, EventValues, NewAgent, NewDraft, Store},
    time,
    world::{
        AccountState, Answer, ApprovalAction, ApprovalStatus, Condition, DraftKind, FolderKind,
        Identity, Owner, Place, RuleAction, Snooze, TASK_AGENTS, TASK_REPLIES, TONES, World,
        paragraphs,
    },
};

#[derive(Clone)]
pub enum MoveTo {
    Inbox,
    Folder(String),
}

/// The event the drawer saves.
#[derive(Clone)]
pub enum EventToSave {
    Saved(String),
    /// The unsaved new event.
    New,
}

#[derive(Clone)]
pub enum ComposeKind {
    New,
    Reply(String),
    ReplyAll(String),
    Forward(String),
}

/// A change the user asked for.
#[derive(Clone)]
pub enum Action {
    SignIn(Identity),
    CancelSignIn,
    SignOut,
    DeleteUser,
    /// "I've signed in" on a provider's sign-in step.
    ConfirmConnect,
    ConnectImap {
        provider: MailProvider,
        address: String,
        password: String,
        server: String,
        port: String,
    },
    Disconnect(String),
    Reconnect(String),
    SetSignature(String, Option<String>),
    Archive(String),
    Trash(String),
    Move(String, MoveTo),
    Star(String, bool),
    Unread(String, bool),
    Snooze(String, Snooze),
    CreateFolder(String),
    Undo,
    DismissNotice,
    Compose(ComposeKind),
    UpdateDraft {
        draft: String,
        to: String,
        subject: String,
        body: String,
    },
    DraftAccount(String, String),
    Attach(String, String, i64),
    /// Draft and attachment.
    RemoveAttachment(String, String),
    Send(String),
    SendLater(String, String, u32),
    /// Takes back a send the Backend holds.
    CancelSend(String),
    Discard(String),
    ComposeModel(String),
    Tone(String),
    WriteReply(String, String),
    UseReply(String),
    SendReply(String),
    ClearReply(String),
    Answer(String, Answer),
    SaveEvent {
        event: EventToSave,
        title: String,
        day: String,
        start: u32,
        end: u32,
        calendar: String,
        guests: Vec<String>,
        /// `None` keeps the saved place.
        place: Option<String>,
    },
    DeleteEvent(String),
    CalendarShown(String, bool),
    Run(String),
    AgentActive(String, bool),
    AgentInstructions(String, String),
    AddTime(String, u32),
    RemoveTime(String, u32),
    AgentModel(String, String),
    /// Agent, permission id, level.
    AgentPermission(String, String, PermissionLevel),
    AgentAccount(String, String, bool),
    AddTopic(String),
    RemoveTopic(String),
    RemoveAgent(String),
    Builder(String, String),
    Idea(String),
    BuilderTrigger(String, bool),
    /// Trigger, filter and choice, by id.
    TriggerFilter(String, String, String),
    /// A schedule choice, by id.
    BuilderSchedule(String),
    /// The custom interval's minutes, as typed.
    BuilderInterval(String),
    BuilderAccount(String, bool),
    BuilderPermission(String, PermissionLevel),
    AskNewRecipient(bool),
    DryRun,
    CancelDryRun,
    CreateAgent,
    DiscardBuilder,
    Chat(String, String),
    ClearChat(String),
    Approve(String),
    Decline(String),
    EditApproval(String),
    Release(String),
    DeleteQuarantined(String),
    AddRule(Condition, Vec<RuleAction>),
    RuleEnabled(String, bool),
    RemoveRule(String),
    Unblock(String),
    SetKey(ModelProvider, String),
    RemoveKey(ModelProvider),
    /// Task id, model name.
    TaskModel(String, String),
    /// A daily token budget as typed, or none.
    Budget(Option<String>),
    Notify(bool),
    Paused(bool),
    DefaultPermission(String, PermissionLevel),
    Export,
    CancelExport,
    Reset,
}

pub struct Effects {
    pub work: Vec<Work>,
    pub navigate: Vec<navigation::Event>,
}

pub struct Context<'a> {
    pub store: &'a Store,
    pub world: &'a World,
    pub navigation: &'a Navigation,
    pub live: &'a mut Live,
    /// The Backend's now, Unix seconds.
    pub now: i64,
    /// The device's now, milliseconds: when outcomes and notices run out.
    pub now_ms: i64,
}

// The pure parts.

/// The send-later menu: (label, day, minute).
pub fn send_later_choices(now: i64) -> Vec<(String, Date, u32)> {
    let today = time::date(now);
    let monday = today.add_days(7 - today.weekday() as i64);
    vec![
        ("Tomorrow morning".into(), today.add_days(1), 8 * 60),
        ("Tomorrow afternoon".into(), today.add_days(1), 13 * 60),
        ("Monday morning".into(), monday, 8 * 60),
    ]
}

/// Seconds from minutes typed as a whole number: "45".
pub fn typed_minutes(text: &str) -> Option<u32> {
    let text = text.trim();
    if text.is_empty() || !text.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    text.parse::<u32>().ok()?.checked_mul(60)
}

/// Tokens typed as a number, grouped or not: "500,000", "500 000", "500000".
pub fn typed_tokens(text: &str) -> Option<u64> {
    let digits: String = text
        .chars()
        .filter(|c| !matches!(c, ',' | '_' | ' ' | '\u{a0}' | '\u{202f}'))
        .collect();
    if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
}

pub fn folder_problem(name: &str, taken: &[String]) -> Option<String> {
    let name = name.trim();
    if name.is_empty() {
        Some("Name the folder".into())
    } else if name.chars().count() > 40 {
        Some("Keep it under 40 characters".into())
    } else {
        taken
            .iter()
            .find(|t| t.to_lowercase() == name.to_lowercase())
            .map(|t| format!("A folder named {t} exists"))
    }
}

/// A condition as it is kept: trimmed and in lower case, a domain without its "@", words split at
/// commas and none empty.
pub fn normalized(condition: Condition) -> Condition {
    let clean = |text: &str| text.trim().to_lowercase();
    match condition {
        Condition::SenderContains(text) => Condition::SenderContains(clean(&text)),
        Condition::SenderDomain(domain) => {
            Condition::SenderDomain(clean(&domain).trim_start_matches('@').into())
        }
        Condition::SubjectContains(words) => Condition::SubjectContains(
            words
                .iter()
                .flat_map(|w| w.split(','))
                .map(clean)
                .filter(|w| !w.is_empty())
                .collect(),
        ),
        Condition::Newsletter | Condition::NewSenderWithLink => condition,
    }
}

/// Why a rule cannot be added. `condition` is normalized.
pub fn rule_problem(
    condition: &Condition,
    actions: &[RuleAction],
    world: &World,
) -> Option<&'static str> {
    let blank = match condition {
        Condition::SenderContains(text) | Condition::SenderDomain(text) => text.is_empty(),
        Condition::SubjectContains(words) => words.is_empty(),
        Condition::Newsletter | Condition::NewSenderWithLink => false,
    };
    let unknown = actions.iter().any(|a| match a {
        RuleAction::MoveTo(folder) => !world
            .folders
            .iter()
            .any(|f| f.id == *folder && f.kind == FolderKind::User),
        RuleAction::HandTo(agent) => world.agent(agent).is_none(),
        RuleAction::Archive
        | RuleAction::Trash
        | RuleAction::MarkImportant
        | RuleAction::KeepForBriefing => false,
    });
    if blank {
        Some("Say what the condition looks for")
    } else if actions.is_empty() {
        Some("Pick what it does")
    } else if unknown {
        Some("Pick one of your folders or agents")
    } else if world
        .settings
        .rules
        .iter()
        .any(|r| r.condition == *condition && r.actions == actions)
    {
        Some("That rule exists")
    } else {
        None
    }
}

pub fn imap_problem(
    provider: MailProvider,
    address: &str,
    password: &str,
    server: &str,
    port: &str,
) -> Option<&'static str> {
    let domain = address.trim().split_once('@').map(|(_, d)| d);
    if !domain.is_some_and(|d| d.contains('.') && !d.starts_with('.') && !d.ends_with('.')) {
        Some("Enter the whole address, like ali@icloud.com")
    } else if password.trim().is_empty() {
        Some("Enter the app password")
    } else if provider == MailProvider::OtherImap && server.trim().is_empty() {
        Some("Enter the IMAP server")
    } else if provider == MailProvider::OtherImap && port.trim().parse::<u16>().is_err() {
        Some("The port is a number, like 993")
    } else {
        None
    }
}

pub fn event_problem(title: &str, start: u32, end: u32) -> Option<&'static str> {
    if title.trim().is_empty() {
        Some("Give it a title")
    } else if end <= start || end > 24 * 60 {
        Some("It has to end after it starts, the same day")
    } else {
        None
    }
}

/// "Mira" from "Mira <mira@laurentian.ca>" or "mira@laurentian.ca"
pub fn first_name(recipients: &str) -> String {
    let first = recipients.split(',').next().unwrap_or("").trim();
    let name = first.split('<').next().unwrap_or("").trim();
    let name = if name.is_empty() { first } else { name };
    let local = name.split('@').next().unwrap_or(name);
    let mut chars = local.chars();
    chars
        .next()
        .map_or(String::new(), |c| c.to_uppercase().chain(chars).collect())
}

/// What the outcome of an answer says; `None` for no answer.
fn answered(answer: Answer) -> Option<&'static str> {
    match answer {
        Answer::Accepted => Some("Accepted"),
        Answer::Tentative => Some("Maybe"),
        Answer::Declined => Some("Declined"),
        Answer::Unanswered => None,
    }
}

// Applying actions.

impl Context<'_> {
    fn done(&mut self, target: Target, text: &str) {
        self.live.done(target, text, self.now_ms);
    }

    fn failed(&mut self, target: Target, reason: &str) {
        self.live.failed(target, reason, self.now_ms);
    }

    /// The draft of a reply to `message`, opened again if it is saved.
    fn reply_draft(
        &self,
        message: &str,
        kind: DraftKind,
        body: &str,
    ) -> store::Result<Option<String>> {
        let w = self.world;
        if let Some(saved) = w
            .drafts
            .iter()
            .find(|d| d.send.is_none() && d.kind == kind && d.message.as_deref() == Some(message))
            && body.is_empty()
        {
            return Ok(Some(saved.id.clone()));
        }
        let Some((thread, m)) = w.message(message) else {
            return Ok(None);
        };
        let account = w.account(&thread.account);
        let signature = account
            .and_then(|a| a.signature.as_deref())
            .map_or(String::new(), |s| format!("\n\n{s}"));
        let to = match kind {
            DraftKind::Reply if m.from_you => thread.from.clone(),
            DraftKind::Reply => m.sender.clone(),
            DraftKind::ReplyAll => std::iter::once(if m.from_you {
                thread.from.clone()
            } else {
                m.sender.clone()
            })
            .chain(thread.others.iter().cloned())
            .collect::<Vec<_>>()
            .join(", "),
            DraftKind::New | DraftKind::Forward => String::new(),
        };
        let (subject, text) = match kind {
            DraftKind::Forward => (
                format!("Fwd: {}", thread.subject),
                format!(
                    "{signature}\n\nForwarded message from {}:\n\n{}",
                    m.sender, m.body
                ),
            ),
            DraftKind::New | DraftKind::Reply | DraftKind::ReplyAll => (
                format!("Re: {}", thread.subject),
                format!("{body}{signature}"),
            ),
        };
        let id = self.store.create_draft(
            &NewDraft {
                kind,
                thread: Some(&thread.id),
                message: Some(message),
                approval: None,
                account: &thread.account,
                recipients: &to,
                subject: &subject,
                body: text.trim_start_matches('\n'),
            },
            self.now,
        )?;
        Ok(Some(id))
    }

    /// Hands a draft to the Backend, now or later, which answers with the send's state.
    fn send(&mut self, draft: &str, at: SendAt, fx: &mut Effects) {
        let Some(d) = self.world.draft(draft).filter(|d| d.send.is_none()) else {
            return;
        };
        if d.recipients.trim().is_empty() {
            self.failed(Target::Draft(draft.into()), "Add a recipient");
            return;
        }
        fx.navigate.push(navigation::Event::CloseComposer);
        fx.work.push(Work::Send {
            draft: draft.into(),
            at,
        });
    }

    /// Moves a thread with undo; the open thread closes.
    fn relocate(
        &mut self,
        thread: &str,
        text: String,
        fx: &mut Effects,
        apply: impl FnOnce(&Store) -> store::Result<()>,
    ) -> store::Result<()> {
        let Some(t) = self.world.thread(thread) else {
            return Ok(());
        };
        let undo = Undo::Thread {
            thread: thread.into(),
            place: t.place,
            snoozed_until: t.snoozed_until,
            folders: t.folders.clone(),
        };
        apply(self.store)?;
        self.live.undo(text, undo, self.now_ms + live::UNDO_MS);
        fx.navigate.extend(leave(self.navigation, thread));
        Ok(())
    }

    /// The accounts an owner would see with `account` in or out.
    fn accounts(&self, owner: Owner, account: String, included: bool) -> Option<Work> {
        let (seen, _) = self.world.grants(&owner)?;
        self.world.account(&account)?;
        let mut accounts: Vec<String> = seen.iter().filter(|a| **a != account).cloned().collect();
        if included {
            accounts.push(account);
        }
        Some(Work::Accounts { owner, accounts })
    }

    pub fn act(&mut self, action: Action) -> store::Result<Effects> {
        let mut fx = Effects {
            work: Vec::new(),
            navigate: Vec::new(),
        };
        let w = self.world;
        let s = self.store;
        let now = self.now;
        match action {
            Action::SignIn(provider) => {
                if !matches!(self.live.sign_in, SignIn::Waiting(_)) {
                    self.live.sign_in = SignIn::Waiting(provider);
                    fx.work.push(Work::SignIn(provider));
                }
            }
            Action::CancelSignIn => self.cancel(Request::SignIn, &mut fx),
            Action::SignOut => {
                s.sign_out()?;
                self.live.clear();
                fx.navigate.push(navigation::Event::Refresh);
            }
            Action::DeleteUser => fx.work.push(Work::DeleteUser),
            Action::ConfirmConnect => {
                if let Screen::Connect(ConnectStep::Connecting(provider)) = self.navigation.screen
                    && matches!(provider, MailProvider::Gmail | MailProvider::Outlook)
                {
                    fx.work.push(Work::Connect {
                        provider,
                        address: String::new(),
                        password: String::new(),
                    });
                    fx.navigate.push(navigation::Event::ConfirmConnect);
                }
            }
            Action::ConnectImap {
                provider,
                address,
                password,
                server,
                port,
            } => match imap_problem(provider, &address, &password, &server, &port) {
                Some(problem) => {
                    self.live.imap = ImapCheck::Rejected(problem.into());
                    self.failed(Target::Form(Form::Imap), problem);
                }
                None => {
                    self.live.imap = ImapCheck::Checking;
                    fx.work.push(Work::Connect {
                        provider,
                        address: address.trim().into(),
                        password,
                    });
                }
            },
            Action::Disconnect(account) => {
                if let Some(a) = w.account(&account) {
                    s.remove_account(&account)?;
                    self.live
                        .info(format!("{} disconnected", a.name), self.now_ms);
                    fx.navigate.push(navigation::Event::Refresh);
                }
            }
            Action::Reconnect(account) => {
                if w.account(&account).is_some() {
                    fx.work.push(Work::Reconnect(account));
                }
            }
            Action::SetSignature(account, signature) => {
                s.set_signature(
                    &account,
                    signature.as_deref().filter(|t| !t.trim().is_empty()),
                )?;
                self.done(Target::Account(account), "Saved");
            }
            Action::Archive(thread) => {
                self.relocate(&thread, "Archived".into(), &mut fx, |s| {
                    s.set_place(&thread, Place::Archive)
                })?;
            }
            Action::Trash(thread) => {
                self.relocate(&thread, "Moved to trash".into(), &mut fx, |s| {
                    s.set_place(&thread, Place::Trash)
                })?;
            }
            Action::Move(thread, MoveTo::Inbox) => {
                self.relocate(&thread, "Moved to Inbox".into(), &mut fx, |s| {
                    s.set_place(&thread, Place::Inbox)
                })?;
            }
            Action::Move(thread, MoveTo::Folder(folder)) => {
                if let Some(f) = w
                    .folders
                    .iter()
                    .find(|f| f.id == folder && f.kind == FolderKind::User)
                {
                    let text = format!("Moved to {}", f.name);
                    self.relocate(&thread, text, &mut fx, |s| {
                        s.move_to_folder(&thread, &folder)
                    })?;
                }
            }
            Action::Star(message, starred) => s.set_starred(&message, starred)?,
            Action::Unread(thread, unread) => s.set_unread(&thread, unread)?,
            Action::Snooze(thread, until) => {
                if w.thread(&thread).is_some() {
                    fx.navigate.extend(leave(self.navigation, &thread));
                    fx.work.push(Work::Snooze { thread, until });
                }
            }
            Action::CreateFolder(name) => {
                let taken: Vec<String> = w.folders.iter().map(|f| f.name.clone()).collect();
                match folder_problem(&name, &taken) {
                    Some(problem) => self.failed(Target::Form(Form::NewFolder), &problem),
                    None => {
                        let id = s.next_id("folders", "folder")?;
                        s.create_folder(&id, name.trim())?;
                        self.done(Target::Form(Form::NewFolder), "Added");
                        self.live.revisions.revise(Field::NewFolder);
                        fx.navigate
                            .push(navigation::Event::Conceal(Disclosure::NewFolder));
                    }
                }
            }
            Action::Undo => match std::mem::replace(&mut self.live.notice, live::Notice::None) {
                live::Notice::Undo {
                    undo:
                        Undo::Thread {
                            thread,
                            place,
                            snoozed_until,
                            folders,
                        },
                    ..
                } => s.restore_thread(&thread, place, snoozed_until, &folders)?,
                live::Notice::Undo {
                    undo: Undo::Send(draft),
                    ..
                } => fx.work.push(Work::CancelSend(draft)),
                notice @ (live::Notice::None | live::Notice::Info { .. }) => {
                    self.live.notice = notice;
                }
            },
            Action::DismissNotice => self.live.notice = live::Notice::None,
            Action::Compose(kind) => {
                let draft = match kind {
                    ComposeKind::New => {
                        let account = w
                            .accounts
                            .iter()
                            .find(|a| a.state == AccountState::Synced)
                            .or(w.accounts.first());
                        match account {
                            Some(a) => {
                                let body = a
                                    .signature
                                    .as_deref()
                                    .map_or(String::new(), |s| format!("\n\n{s}"));
                                Some(s.create_draft(
                                    &NewDraft {
                                        kind: DraftKind::New,
                                        thread: None,
                                        message: None,
                                        approval: None,
                                        account: &a.id,
                                        recipients: "",
                                        subject: "",
                                        body: &body,
                                    },
                                    now,
                                )?)
                            }
                            None => None,
                        }
                    }
                    ComposeKind::Reply(m) => self.reply_draft(&m, DraftKind::Reply, "")?,
                    ComposeKind::ReplyAll(m) => self.reply_draft(&m, DraftKind::ReplyAll, "")?,
                    ComposeKind::Forward(m) => self.reply_draft(&m, DraftKind::Forward, "")?,
                };
                fx.navigate.extend(draft.map(navigation::Event::OpenDraft));
            }
            Action::UpdateDraft {
                draft,
                to,
                subject,
                body,
            } => {
                if w.draft(&draft).is_some_and(|d| d.send.is_none()) {
                    s.update_draft(&draft, &to, &subject, &body, now)?;
                }
            }
            Action::DraftAccount(draft, account) => {
                if w.account(&account).is_some() {
                    s.set_draft_account(&draft, &account)?;
                }
            }
            Action::Attach(draft, name, size) => {
                if w.draft(&draft).is_some() && !name.trim().is_empty() {
                    s.attach(&draft, name.trim(), size)?;
                }
            }
            Action::RemoveAttachment(draft, attachment) => {
                s.remove_attachment(&draft, &attachment)?;
            }
            Action::Send(draft) => self.send(&draft, SendAt::Now, &mut fx),
            Action::SendLater(draft, day, minute) => match Date::parse(&day) {
                Some(day) if minute < 24 * 60 => {
                    self.send(&draft, SendAt::Later(day, minute), &mut fx);
                }
                Some(_) | None => self.failed(Target::Draft(draft), "Pick a day and a time"),
            },
            Action::CancelSend(draft) => {
                if w.draft(&draft)
                    .and_then(|d| d.send.as_ref())
                    .is_some_and(|s| s.held())
                {
                    fx.work.push(Work::CancelSend(draft));
                }
            }
            Action::Discard(draft) => {
                s.discard_draft(&draft)?;
                fx.navigate.push(navigation::Event::CloseComposer);
                self.live.info("Draft discarded".into(), self.now_ms);
            }
            Action::ComposeModel(model) => {
                if w.model(&model).is_some() {
                    s.set_task_model(TASK_REPLIES, &model)?;
                }
            }
            Action::Tone(tone) => {
                if TONES.contains(&tone.as_str()) {
                    s.set_compose_tone(&tone)?;
                }
            }
            Action::WriteReply(thread, instruction) => {
                let writing = matches!(self.live.replies.get(&thread), Some(Generation::Writing));
                if !w.ai_on() {
                    self.failed(
                        Target::Thread(thread),
                        "AI compose is off until you add a model key",
                    );
                } else if w.thread(&thread).is_some() && !writing {
                    self.live
                        .replies
                        .insert(thread.clone(), Generation::Writing);
                    self.live.revisions.revise(Field::Reply(thread.clone()));
                    fx.work.push(Work::WriteReply {
                        thread,
                        instruction,
                    });
                }
            }
            Action::UseReply(thread) | Action::SendReply(thread)
                if !matches!(self.live.replies.get(&thread), Some(Generation::Written(_))) => {}
            Action::UseReply(thread) => {
                if let Some(draft) = self.written_reply(&thread)? {
                    fx.navigate.push(navigation::Event::OpenDraft(draft));
                }
            }
            Action::SendReply(thread) => {
                if let Some(draft) = self.written_reply(&thread)? {
                    let world = self.store.load(now, None)?;
                    let mut cx = Context {
                        store: s,
                        world: &world,
                        navigation: self.navigation,
                        live: self.live,
                        now,
                        now_ms: self.now_ms,
                    };
                    cx.send(&draft, SendAt::Now, &mut fx);
                }
            }
            Action::ClearReply(thread) => self.cancel(Request::Reply(thread), &mut fx),
            Action::Answer(event, answer) => {
                if w.event(&event).is_some_and(|e| e.invitation.is_some())
                    && let Some(text) = answered(answer)
                {
                    s.answer_invitation(&event, answer)?;
                    self.done(Target::Event(event.clone()), text);
                    fx.navigate
                        .push(navigation::Event::Conceal(Disclosure::Invitation(event)));
                }
            }
            Action::SaveEvent {
                event,
                title,
                day,
                start,
                end,
                calendar,
                guests,
                place,
            } => {
                let problem = event_problem(&title, start, end)
                    .or_else(|| Date::parse(&day).is_none().then_some("Pick a day"))
                    .or_else(|| w.calendar(&calendar).is_none().then_some("Pick a calendar"));
                // The unsaved event is saved under a new id; a saved one must still exist.
                let (event, target, new) = match event {
                    EventToSave::Saved(id) if w.event(&id).is_some() => {
                        (id.clone(), Target::Event(id), false)
                    }
                    EventToSave::Saved(_) => return Ok(fx),
                    EventToSave::New if self.navigation.event == navigation::Drawer::New => (
                        s.next_id("events", "event")?,
                        Target::Form(Form::NewEvent),
                        true,
                    ),
                    EventToSave::New => return Ok(fx),
                };
                match problem {
                    Some(problem) => self.failed(target, problem),
                    None => {
                        let guests: Vec<String> = guests
                            .iter()
                            .map(|g| g.trim().to_string())
                            .filter(|g| !g.is_empty())
                            .collect();
                        s.save_event(
                            &event,
                            &EventValues {
                                title: title.trim(),
                                day: &day,
                                start,
                                end,
                                calendar: &calendar,
                                guests: &guests,
                                place: place.as_deref().map(str::trim),
                            },
                            new,
                        )?;
                        self.done(target, "Saved");
                        // The drawer shows the saved event, or closes if its calendar is hidden.
                        fx.navigate.push(navigation::Event::CloseEvent);
                        fx.navigate.push(navigation::Event::SelectEvent(event));
                    }
                }
            }
            Action::DeleteEvent(event) => {
                if let Some(e) = w.event(&event) {
                    s.delete_event(&event)?;
                    self.live.info(format!("{} deleted", e.title), self.now_ms);
                    fx.navigate.push(navigation::Event::CloseEvent);
                }
            }
            Action::CalendarShown(calendar, shown) => s.set_calendar_shown(&calendar, shown)?,
            Action::Run(agent) => {
                if w.agent(&agent).is_some() {
                    fx.work.push(Work::Run(agent));
                }
            }
            Action::AgentActive(agent, active) => s.set_agent_active(&agent, active)?,
            Action::AgentInstructions(agent, text) => s.set_agent_instructions(&agent, &text)?,
            Action::AddTime(agent, minute) => {
                if let Some(Schedule::Times { minutes, .. }) = w.agent(&agent).map(|a| &a.schedule)
                {
                    if minute >= 24 * 60 {
                        self.failed(Target::Agent(agent), "Pick a time of day");
                    } else if minutes.contains(&minute) {
                        let text = format!("It runs at {} already", time::clock(minute));
                        self.failed(Target::Agent(agent), &text);
                    } else {
                        s.add_agent_time(&agent, minute)?;
                    }
                }
            }
            Action::RemoveTime(agent, minute) => {
                if let Some(Schedule::Times { minutes, .. }) = w.agent(&agent).map(|a| &a.schedule)
                {
                    if minutes.len() <= 1 {
                        self.failed(Target::Agent(agent), "It needs one time at least");
                    } else {
                        s.remove_agent_time(&agent, minute)?;
                    }
                }
            }
            Action::AgentModel(agent, model) => {
                if let Some(m) = w.model(&model) {
                    s.set_agent_model(&agent, &m.name, m.provider)?;
                }
            }
            Action::AgentPermission(agent, action, level) => {
                if w.agent(&agent).is_some() {
                    fx.work.push(Work::Permission {
                        owner: Owner::Agent(agent),
                        action,
                        level,
                    });
                }
            }
            Action::AgentAccount(agent, account, included) => {
                fx.work
                    .extend(self.accounts(Owner::Agent(agent), account, included));
            }
            Action::AddTopic(topic) => {
                if topic.trim().is_empty() {
                    self.failed(Target::Form(Form::NewsTopic), "Name a topic");
                } else {
                    fx.work.push(Work::AddTopic(topic.trim().into()));
                }
            }
            Action::RemoveTopic(topic) => s.remove_news_topic(&topic)?,
            Action::RemoveAgent(agent) => {
                if let Some(a) = w.agent(&agent).filter(|a| !a.system()) {
                    s.remove_agent(&agent)?;
                    self.live.info(format!("{} deleted", a.name), self.now_ms);
                    fx.navigate.push(navigation::Event::Open(Route::Agents));
                }
            }
            Action::Builder(name, instructions) => s.update_builder(&name, &instructions)?,
            Action::Idea(idea) => {
                s.update_builder(&w.builder.name, &idea)?;
                self.live.revisions.revise(Field::Builder);
            }
            Action::BuilderTrigger(trigger, chosen) => {
                if builder::trigger(&trigger).is_some() {
                    s.set_builder_trigger(&trigger, chosen)?;
                }
            }
            Action::TriggerFilter(trigger, filter, choice) => {
                let filter = builder::trigger(&trigger)
                    .and_then(|t| t.filters.iter().find(|f| f.id == filter));
                if let Some(filter) = filter
                    && builder::options(filter, w)
                        .iter()
                        .any(|(id, _)| *id == choice)
                {
                    s.set_trigger_filter(&trigger, filter.id, &choice)?;
                }
            }
            Action::BuilderSchedule(choice) => {
                if let Some(option) = builder::schedule_option(&choice) {
                    s.set_builder_schedule(option.id)?;
                }
            }
            Action::BuilderInterval(typed) => match typed_minutes(&typed) {
                Some(seconds) => fx.work.push(Work::Interval(seconds)),
                None => self.failed(
                    Target::Form(Form::BuilderInterval),
                    "Type the minutes as a whole number",
                ),
            },
            Action::BuilderAccount(account, can_see) => {
                fx.work
                    .extend(self.accounts(Owner::Builder, account, can_see));
            }
            Action::BuilderPermission(action, level) => fx.work.push(Work::Permission {
                owner: Owner::Builder,
                action,
                level,
            }),
            Action::AskNewRecipient(ask) => s.set_ask_before_new_recipient(ask)?,
            Action::DryRun => {
                if self.live.dry_run != DryRun::Reading {
                    self.live.dry_run = DryRun::Reading;
                    fx.work.push(Work::DryRun);
                }
            }
            Action::CancelDryRun => self.cancel(Request::DryRun, &mut fx),
            Action::CreateAgent => match builder::incomplete(&w.builder, w) {
                Some(problem) => self.failed(Target::Form(Form::AgentBuilder), &problem),
                None => {
                    let b = &w.builder;
                    let id = s.next_id("agents", "agent")?;
                    let model = w.task_model(TASK_AGENTS).or(w.models.first());
                    s.create_agent(
                        &NewAgent {
                            id: &id,
                            name: b.name.trim(),
                            instructions: b.instructions.trim(),
                            model: model.map_or("", |m| m.name.as_str()),
                            model_provider: model.map_or(ModelProvider::Anthropic, |m| m.provider),
                            schedule: &builder::schedule(b),
                        },
                        now,
                    )?;
                    self.builder_reset(&mut fx);
                    self.live
                        .info(format!("{} is ready", b.name.trim()), self.now_ms);
                    fx.navigate.push(navigation::Event::Open(Route::Agent(
                        id,
                        PaneChoice::Default,
                    )));
                }
            },
            Action::DiscardBuilder => {
                s.reset_builder()?;
                self.builder_reset(&mut fx);
            }
            Action::Chat(agent, text) => {
                if w.agent(&agent).is_some_and(|a| a.has_chat) {
                    // Whether the agent may run, and whether a turn is running, is the Backend's
                    // answer: a turn it will not start ends stopped, with the reason.
                    if text.trim().is_empty() {
                        self.failed(Target::Agent(agent), "Write a message");
                    } else {
                        let turn = s.add_turn(&agent, text.trim(), "Reading your mail", now)?;
                        self.live.revisions.revise(Field::Chat(agent.clone()));
                        fx.work.push(Work::Chat { agent, turn });
                    }
                }
            }
            Action::ClearChat(agent) => match w.agent(&agent) {
                Some(a) if a.running_turn().is_some() => {
                    self.failed(Target::Agent(agent), "Wait for the answer first");
                }
                Some(_) => s.clear_turns(&agent)?,
                None => {}
            },
            Action::Approve(approval) | Action::Decline(approval)
                if w.approval(&approval)
                    .is_none_or(|a| a.status != ApprovalStatus::Waiting) => {}
            Action::Approve(approval) => {
                s.resolving(&approval)?;
                fx.work.push(Work::Resolve {
                    approval,
                    approved: true,
                });
            }
            Action::Decline(approval) => {
                s.resolving(&approval)?;
                fx.work.push(Work::Resolve {
                    approval,
                    approved: false,
                });
            }
            Action::EditApproval(approval) => {
                if let Some(a) = w
                    .approval(&approval)
                    .filter(|a| a.status == ApprovalStatus::Waiting)
                    && let ApprovalAction::Send(thread) = &a.action
                    && let Some(t) = w.thread(thread)
                    && let Some(last) = t.messages.last()
                {
                    let body = paragraphs(&a.draft).join("\n\n");
                    let draft = s.create_draft(
                        &NewDraft {
                            kind: DraftKind::Reply,
                            thread: Some(thread),
                            message: Some(&last.id),
                            approval: Some(&approval),
                            account: &t.account,
                            recipients: &t.from,
                            subject: &format!("Re: {}", t.subject),
                            body: &body,
                        },
                        now,
                    )?;
                    fx.navigate.push(navigation::Event::OpenDraft(draft));
                }
            }
            Action::Release(quarantined) => {
                if w.quarantine.iter().any(|q| q.id == quarantined) {
                    let thread = s.next_id("threads", "thread")?;
                    s.release_quarantined(&quarantined, &thread)?;
                    self.live.info("Moved to your inbox".into(), self.now_ms);
                    fx.navigate.push(navigation::Event::Refresh);
                }
            }
            Action::DeleteQuarantined(quarantined) => {
                s.delete_quarantined(&quarantined)?;
                self.live.info("Deleted".into(), self.now_ms);
                fx.navigate.push(navigation::Event::Refresh);
            }
            Action::AddRule(condition, actions) => {
                let condition = normalized(condition);
                match rule_problem(&condition, &actions, w) {
                    Some(problem) => self.failed(Target::Form(Form::NewRule), problem),
                    None => {
                        let id = s.next_id("rules", "rule")?;
                        s.add_rule(&id, &condition, &actions)?;
                        self.done(Target::Form(Form::NewRule), "Added");
                        self.live.revisions.revise(Field::NewRule);
                        fx.navigate
                            .push(navigation::Event::Conceal(Disclosure::NewRule));
                    }
                }
            }
            Action::RuleEnabled(rule, enabled) => s.set_rule_enabled(&rule, enabled)?,
            Action::RemoveRule(rule) => s.remove_rule(&rule)?,
            Action::Unblock(address) => s.unblock(&address)?,
            Action::SetKey(provider, key) => {
                if key.trim().is_empty() {
                    self.failed(Target::Key(provider), "Paste the key");
                } else {
                    fx.work.push(Work::SetKey(provider, key.trim().into()));
                }
            }
            Action::RemoveKey(provider) => fx.work.push(Work::RemoveKey(provider)),
            Action::TaskModel(task, model) => {
                if w.model(&model).is_some() && w.settings.tasks.iter().any(|t| t.id == task) {
                    s.set_task_model(&task, &model)?;
                }
            }
            Action::Budget(None) => fx.work.push(Work::Budget(None)),
            Action::Budget(Some(typed)) => match typed_tokens(&typed) {
                Some(tokens) => fx.work.push(Work::Budget(Some(tokens))),
                None => self.failed(
                    Target::Form(Form::Budget),
                    "Type the tokens as a number, like 500,000",
                ),
            },
            Action::Notify(notify) => s.set_notify(notify)?,
            Action::Paused(paused) => fx.work.push(Work::Pause(paused)),
            Action::DefaultPermission(action, level) => fx.work.push(Work::Permission {
                owner: Owner::Defaults,
                action,
                level,
            }),
            Action::Export => {
                if !matches!(self.live.export, Export::Running(_)) {
                    self.live.export = Export::Running(0);
                    fx.work.push(Work::Export);
                }
            }
            Action::CancelExport => self.cancel(Request::Export, &mut fx),
            Action::Reset => {
                s.reset()?;
                self.live.clear();
                self.live.info("Local data reset".into(), self.now_ms);
                fx.work.push(Work::Sync);
                fx.navigate.push(navigation::Event::Open(Route::Inbox(
                    navigation::Listing::ALL,
                )));
            }
        }
        Ok(fx)
    }

    /// Stops waiting for a request, if it is waited on.
    fn cancel(&mut self, request: Request, fx: &mut Effects) {
        self.live.cancel(&request);
        fx.work.push(Work::Cancel(request));
    }

    /// The builder started over: it sees every account, with the levels the Backend offers.
    fn builder_reset(&mut self, fx: &mut Effects) {
        self.live.dry_run = DryRun::Idle;
        self.live.revisions.revise(Field::Builder);
        fx.work.push(Work::Accounts {
            owner: Owner::Builder,
            accounts: self.world.accounts.iter().map(|a| a.id.clone()).collect(),
        });
    }

    /// The written reply as a saved draft.
    fn written_reply(&mut self, thread: &str) -> store::Result<Option<String>> {
        let Some(Generation::Written(text)) = self.live.replies.get(thread).cloned() else {
            return Ok(None);
        };
        let Some(last) = self.world.thread(thread).and_then(|t| t.messages.last()) else {
            return Ok(None);
        };
        let draft = self.reply_draft(&last.id, DraftKind::Reply, &text)?;
        self.live.replies.remove(thread);
        Ok(draft)
    }
}

/// Leaving a thread that is open: back to its list.
fn leave(navigation: &Navigation, thread: &str) -> Option<navigation::Event> {
    matches!(&navigation.screen, Screen::Thread { id, .. } if id == thread)
        .then_some(navigation::Event::Back)
}

/// The selection a navigation holds, if any.
pub fn selected(selection: &Selection) -> Option<&str> {
    match selection {
        Selection::Id(id) => Some(id),
        Selection::None => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::tests::{NOW, fixture};

    /// 2026-11-11 is a Wednesday.
    fn at(day: &str, minute: u32) -> i64 {
        time::at(Date::parse(day).expect(day), minute)
    }

    #[test]
    fn send_later_offers_monday_of_next_week() {
        let now = at("2026-11-11", 9 * 60 + 50);
        let choices = send_later_choices(now);
        assert_eq!(choices[2].1, Date::parse("2026-11-16").unwrap());
    }

    #[test]
    fn form_checks() {
        let taken = vec!["Receipts".to_string()];
        assert_eq!(
            folder_problem("  ", &taken).as_deref(),
            Some("Name the folder")
        );
        assert_eq!(
            folder_problem("receipts", &taken).as_deref(),
            Some("A folder named Receipts exists")
        );
        assert_eq!(folder_problem("Travel", &taken), None);
        assert_eq!(
            event_problem("Lunch", 600, 600),
            Some("It has to end after it starts, the same day")
        );
    }

    #[test]
    fn rules_are_checked_as_structured_data() {
        let w = fixture().load(NOW, None).unwrap();
        let domain = normalized(Condition::SenderDomain(" @Fernhill.ORG ".into()));
        assert_eq!(domain, Condition::SenderDomain("fernhill.org".into()));
        assert_eq!(
            rule_problem(&domain, &[RuleAction::MarkImportant], &w),
            Some("That rule exists")
        );
        assert_eq!(
            rule_problem(&domain, &[RuleAction::Archive], &w),
            None,
            "other actions make another rule"
        );
        assert_eq!(
            rule_problem(
                &Condition::SenderContains(String::new()),
                &[RuleAction::Archive],
                &w
            ),
            Some("Say what the condition looks for")
        );
        assert_eq!(
            rule_problem(&Condition::Newsletter, &[], &w),
            Some("Pick what it does")
        );
        assert_eq!(
            rule_problem(
                &Condition::Newsletter,
                &[RuleAction::MoveTo("inbox".into())],
                &w
            ),
            Some("Pick one of your folders or agents")
        );
        let words = normalized(Condition::SubjectContains(vec![
            " Invoice, receipt".into(),
            " ".into(),
        ]));
        assert_eq!(
            words,
            Condition::SubjectContains(vec!["invoice".into(), "receipt".into()])
        );
    }

    #[test]
    fn imap_checks() {
        use MailProvider::{Icloud, OtherImap};
        let p = |provider, address, password, server, port| {
            imap_problem(provider, address, password, server, port)
        };
        assert_eq!(
            p(Icloud, "ali", "x", "", ""),
            Some("Enter the whole address, like ali@icloud.com")
        );
        assert_eq!(
            p(Icloud, "ali@icloud", "x", "", ""),
            Some("Enter the whole address, like ali@icloud.com")
        );
        assert_eq!(
            p(Icloud, "ali@icloud.com", " ", "", ""),
            Some("Enter the app password")
        );
        assert_eq!(p(Icloud, "ali@icloud.com", "pw", "", ""), None);
        assert_eq!(
            p(OtherImap, "a@b.org", "pw", "", "993"),
            Some("Enter the IMAP server")
        );
        assert_eq!(
            p(OtherImap, "a@b.org", "pw", "imap.b.org", "x"),
            Some("The port is a number, like 993")
        );
        assert_eq!(p(OtherImap, "a@b.org", "pw", "imap.b.org", "993"), None);
    }

    #[test]
    fn typed_minutes_and_tokens_are_parsed_by_shape_only() {
        assert_eq!(typed_minutes(" 45 "), Some(2_700));
        assert_eq!(
            typed_minutes("5"),
            Some(300),
            "the Backend judges the length"
        );
        for bad in ["", "1.5", "-5", "ten", "99999999"] {
            assert_eq!(typed_minutes(bad), None, "{bad}");
        }
        assert_eq!(typed_tokens("500,000"), Some(500_000));
        assert_eq!(typed_tokens("500 000"), Some(500_000));
        assert_eq!(typed_tokens("1000"), Some(1_000));
        for bad in ["", ",", "5k", "-1", "1.5"] {
            assert_eq!(typed_tokens(bad), None, "{bad}");
        }
    }

    #[test]
    fn first_names_from_recipients() {
        assert_eq!(first_name("Mira <mira@laurentian.ca>, Priya"), "Mira");
        assert_eq!(first_name("mira@laurentian.ca"), "Mira");
        assert_eq!(first_name(""), "");
    }
}
