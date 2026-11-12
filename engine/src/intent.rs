//! The one dispatch from the `Intent` oneof to what each intent changes. Unknown and unspecified
//! enum values and unset oneofs are decode errors.

use crate::{
    action::{Action, ComposeKind, EventToSave, MoveTo},
    backend::Update,
    connection,
    navigation::{self, Disclosure, Listing, Mailbox, Route},
    preferences::Preference,
    proto::ui::v1::{
        AgentDetailPane, BuilderStep, CalendarMode, Density, DisclosureTarget, Form, InboxCategory,
        InboxScope, InvitationReply, MailProvider, ModelProvider, PermissionLevel,
        RuleAction as UiAction, RuleCondition, SettingsSection, Sidebar, SignInProvider,
        SnoozePreset, Theme, disclosure_target::Target, inbox_scope::Scope, intent::Kind,
        move_thread::To, open_composer::Kind as Compose, rule_action::Action as Act,
        rule_condition::Condition as When, save_event::Event as Saved, set_budget::Budget,
        set_signature::Signature,
    },
    world::{Answer, Condition, Identity, RuleAction, Snooze},
};

#[derive(Debug)]
pub struct InvalidIntent;

impl std::fmt::Display for InvalidIntent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("bytes are not a ui.v1.Intent with its oneofs set and its enums specified")
    }
}

impl std::error::Error for InvalidIntent {}

/// Everything the engine handles, one at a time: the user's intents, the Backend's updates, the
/// connection check's results and the timer.
pub enum Input {
    Connection(connection::Event),
    Navigation(navigation::Event),
    Preference(Preference),
    Action(Action),
    Update(Update),
    /// The timer went off: outcomes and notices run out.
    Tick,
}

/// An enum value the Frontend sent: unknown and unspecified values are decode errors.
fn known<E: Copy + Into<i32>>(
    value: Result<E, prost::UnknownEnumValue>,
) -> Result<E, InvalidIntent> {
    match value {
        Ok(value) if value.into() != 0 => Ok(value),
        Ok(_) | Err(_) => Err(InvalidIntent),
    }
}

/// A navigation enum the Frontend sent, as the engine's: unknown and unspecified values are
/// decode errors.
fn engine<P, E>(value: Result<P, prost::UnknownEnumValue>) -> Result<E, InvalidIntent>
where
    E: TryFrom<P>,
{
    value
        .ok()
        .and_then(|v| E::try_from(v).ok())
        .ok_or(InvalidIntent)
}

fn disclosure(target: Option<DisclosureTarget>) -> Result<Disclosure, InvalidIntent> {
    Ok(match target.and_then(|t| t.target).ok_or(InvalidIntent)? {
        Target::Form(form) => match known(Form::try_from(form))? {
            Form::NewFolder => Disclosure::NewFolder,
            Form::NewRule => Disclosure::NewRule,
            Form::Budget => Disclosure::Budget,
            _ => return Err(InvalidIntent),
        },
        Target::Key(provider) => Disclosure::Key(known(ModelProvider::try_from(provider))?),
        Target::EventId(event) => Disclosure::Invitation(event),
        Target::RunId(run) => Disclosure::Run(run),
    })
}

fn listing(scope: Option<InboxScope>) -> Result<Listing, InvalidIntent> {
    Ok(match scope.and_then(|s| s.scope).ok_or(InvalidIntent)? {
        Scope::All(_) => Listing::ALL,
        Scope::AccountId(id) => Listing::Mailbox(Mailbox::Account(id), InboxCategory::ForYou),
        Scope::FolderId(id) => Listing::Folder(id),
    })
}

fn level(value: i32) -> Result<PermissionLevel, InvalidIntent> {
    known(PermissionLevel::try_from(value))
}

fn identity(value: i32) -> Result<Identity, InvalidIntent> {
    match known(SignInProvider::try_from(value))? {
        SignInProvider::Google => Ok(Identity::Google),
        SignInProvider::Microsoft => Ok(Identity::Microsoft),
        SignInProvider::Unspecified => Err(InvalidIntent),
    }
}

fn snooze(value: i32) -> Result<Snooze, InvalidIntent> {
    match known(SnoozePreset::try_from(value))? {
        SnoozePreset::LaterToday => Ok(Snooze::LaterToday),
        SnoozePreset::Tomorrow => Ok(Snooze::Tomorrow),
        SnoozePreset::ThisWeekend => Ok(Snooze::ThisWeekend),
        SnoozePreset::NextWeek => Ok(Snooze::NextWeek),
        SnoozePreset::Unspecified => Err(InvalidIntent),
    }
}

fn answer(value: i32) -> Result<Answer, InvalidIntent> {
    match known(InvitationReply::try_from(value))? {
        InvitationReply::Accept => Ok(Answer::Accepted),
        InvitationReply::Maybe => Ok(Answer::Tentative),
        InvitationReply::Decline => Ok(Answer::Declined),
        InvitationReply::Unspecified => Err(InvalidIntent),
    }
}

fn condition(condition: Option<RuleCondition>) -> Result<Condition, InvalidIntent> {
    Ok(
        match condition.and_then(|c| c.condition).ok_or(InvalidIntent)? {
            When::SenderContains(text) => Condition::SenderContains(text),
            When::SenderDomain(domain) => Condition::SenderDomain(domain),
            When::SubjectContains(words) => Condition::SubjectContains(words.words),
            When::Newsletter(_) => Condition::Newsletter,
            When::NewSenderWithLink(_) => Condition::NewSenderWithLink,
        },
    )
}

fn rule_action(action: UiAction) -> Result<RuleAction, InvalidIntent> {
    Ok(match action.action.ok_or(InvalidIntent)? {
        Act::MoveToFolderId(folder) => RuleAction::MoveTo(folder),
        Act::Archive(_) => RuleAction::Archive,
        Act::Trash(_) => RuleAction::Trash,
        Act::MarkImportant(_) => RuleAction::MarkImportant,
        Act::HandToAgentId(agent) => RuleAction::HandTo(agent),
        Act::KeepForBriefing(_) => RuleAction::KeepForBriefing,
    })
}

pub fn input(kind: Kind) -> Result<Input, InvalidIntent> {
    use navigation::Event as Nav;
    let open = |route| Ok(Input::Navigation(Nav::Open(route)));
    let nav = |event| Ok(Input::Navigation(event));
    let act = |action| Ok(Input::Action(action));
    match kind {
        Kind::RetryConnection(_) => Ok(Input::Connection(connection::Event::RetryRequested)),
        Kind::SetTheme(set) => Ok(Input::Preference(Preference::Theme(known(
            Theme::try_from(set.theme),
        )?))),
        Kind::SetDensity(set) => Ok(Input::Preference(Preference::Density(known(
            Density::try_from(set.density),
        )?))),
        Kind::SetSidebar(set) => Ok(Input::Preference(Preference::Sidebar(known(
            Sidebar::try_from(set.sidebar),
        )?))),
        Kind::OpenPath(open_path) => nav(Nav::OpenPath(open_path.path)),
        Kind::Back(_) => nav(Nav::Back),
        Kind::ContinueWithProvider(sign_in) => act(Action::SignIn(identity(sign_in.provider)?)),
        Kind::CancelSignIn(_) => act(Action::CancelSignIn),
        Kind::OpenConnect(_) => open(Route::Connect),
        Kind::PickMailProvider(pick) => nav(Nav::PickProvider(known(MailProvider::try_from(
            pick.provider,
        ))?)),
        Kind::ConfirmConnect(_) => act(Action::ConfirmConnect),
        Kind::OpenInbox(inbox) => open(Route::Inbox(listing(inbox.scope)?)),
        Kind::SelectCategory(select) => nav(Nav::SelectCategory(known(InboxCategory::try_from(
            select.category,
        ))?)),
        Kind::SelectThread(select) => nav(Nav::SelectThread(select.thread_id)),
        Kind::OpenThread(thread) => open(Route::Thread(thread.thread_id)),
        Kind::ExpandMessage(message) => nav(Nav::ExpandMessage(message.message_id)),
        Kind::CollapseMessage(message) => nav(Nav::CollapseMessage(message.message_id)),
        Kind::OpenAgents(_) => open(Route::Agents),
        Kind::SelectAgent(select) => nav(Nav::SelectAgent(select.agent_id)),
        Kind::OpenAgentBuilder(_) => open(Route::AgentBuilder),
        Kind::OpenSettings(settings) => open(Route::Settings(engine(SettingsSection::try_from(
            settings.section,
        ))?)),
        Kind::OpenCalendar(_) => open(Route::Calendar(None)),
        Kind::SelectEvent(select) => nav(Nav::SelectEvent(select.event_id)),
        Kind::CloseEvent(_) => nav(Nav::CloseEvent),
        Kind::OpenQuarantine(_) => open(Route::Quarantine),
        Kind::SelectQuarantined(select) => nav(Nav::SelectQuarantined(select.quarantined_id)),
        Kind::OpenBriefing(_) => open(Route::Briefing),
        Kind::SelectEdition(select) => nav(Nav::SelectEdition(select.edition_id)),
        Kind::OpenApprovals(_) => open(Route::Approvals),
        Kind::SelectCalendarView(select) => nav(Nav::SelectCalendarMode(engine(
            CalendarMode::try_from(select.mode),
        )?)),
        Kind::ShowPreviousPeriod(_) => nav(Nav::PreviousPeriod),
        Kind::ShowNextPeriod(_) => nav(Nav::NextPeriod),
        Kind::ShowToday(_) => nav(Nav::Today),
        Kind::SelectMonth(select) => nav(Nav::SelectMonth(select.month_id)),
        Kind::SelectBuilderStep(select) => {
            nav(Nav::SelectStep(engine(BuilderStep::try_from(select.step))?))
        }
        Kind::SelectDay(select) => nav(Nav::SelectDay(select.day_id)),
        Kind::SelectAgentDetailPane(select) => nav(Nav::SelectDetailPane(engine(
            AgentDetailPane::try_from(select.pane),
        )?)),
        Kind::SelectNext(_) => nav(Nav::SelectNext),
        Kind::SelectPrevious(_) => nav(Nav::SelectPrevious),
        Kind::OpenDisclosure(open) => nav(Nav::Disclose(disclosure(open.target)?)),
        Kind::CloseDisclosure(close) => nav(Nav::Conceal(disclosure(close.target)?)),
        Kind::SearchMail(search) => nav(Nav::Search(search.query)),
        Kind::ClearSearch(_) => nav(Nav::Search(String::new())),
        Kind::OpenDraft(open) => nav(Nav::OpenDraft(open.draft_id)),
        Kind::CloseComposer(_) => nav(Nav::CloseComposer),
        Kind::SignOut(_) => act(Action::SignOut),
        Kind::DeleteUserAccount(_) => act(Action::DeleteUser),
        Kind::ConnectImap(c) => act(Action::ConnectImap {
            provider: known(MailProvider::try_from(c.provider))?,
            address: c.address,
            password: c.password,
            server: c.server,
            port: c.port,
        }),
        Kind::DisconnectAccount(d) => act(Action::Disconnect(d.account_id)),
        Kind::ReconnectAccount(r) => act(Action::Reconnect(r.account_id)),
        Kind::SetSignature(s) => act(Action::SetSignature(
            s.account_id,
            match s.signature.ok_or(InvalidIntent)? {
                Signature::Text(text) => Some(text),
                Signature::None(_) => None,
            },
        )),
        Kind::ArchiveThread(a) => act(Action::Archive(a.thread_id)),
        Kind::TrashThread(t) => act(Action::Trash(t.thread_id)),
        Kind::MoveThread(m) => act(Action::Move(
            m.thread_id,
            match m.to.ok_or(InvalidIntent)? {
                To::Inbox(_) => MoveTo::Inbox,
                To::FolderId(folder) => MoveTo::Folder(folder),
            },
        )),
        Kind::SetStarred(s) => act(Action::Star(s.message_id, s.starred)),
        Kind::SetUnread(u) => act(Action::Unread(u.thread_id, u.unread)),
        Kind::SnoozeThread(s) => act(Action::Snooze(s.thread_id, snooze(s.preset)?)),
        Kind::CreateFolder(c) => act(Action::CreateFolder(c.name)),
        Kind::Undo(_) => act(Action::Undo),
        Kind::DismissNotice(_) => act(Action::DismissNotice),
        Kind::OpenComposer(o) => act(Action::Compose(match o.kind.ok_or(InvalidIntent)? {
            Compose::NewMail(_) => ComposeKind::New,
            Compose::ReplyToMessageId(m) => ComposeKind::Reply(m),
            Compose::ReplyAllToMessageId(m) => ComposeKind::ReplyAll(m),
            Compose::ForwardMessageId(m) => ComposeKind::Forward(m),
        })),
        Kind::UpdateDraft(u) => act(Action::UpdateDraft {
            draft: u.draft_id,
            to: u.to,
            subject: u.subject,
            body: u.body,
        }),
        Kind::SetDraftAccount(s) => act(Action::DraftAccount(s.draft_id, s.account_id)),
        Kind::AttachFile(a) => act(Action::Attach(
            a.draft_id,
            a.name,
            i64::try_from(a.size_bytes).map_err(|_| InvalidIntent)?,
        )),
        Kind::RemoveAttachment(r) => act(Action::RemoveAttachment(r.draft_id, r.attachment_id)),
        Kind::SendDraft(s) => act(Action::Send(s.draft_id)),
        Kind::SendDraftLater(s) => act(Action::SendLater(s.draft_id, s.day_id, s.minute)),
        Kind::CancelSend(c) => act(Action::CancelSend(c.draft_id)),
        Kind::DiscardDraft(d) => act(Action::Discard(d.draft_id)),
        Kind::SelectComposeModel(s) => act(Action::ComposeModel(s.model_name)),
        Kind::SelectTone(s) => act(Action::Tone(s.tone)),
        Kind::WriteReply(w) => act(Action::WriteReply(w.thread_id, w.instruction)),
        Kind::UseReply(u) => act(Action::UseReply(u.thread_id)),
        Kind::SendReply(s) => act(Action::SendReply(s.thread_id)),
        Kind::ClearReply(c) => act(Action::ClearReply(c.thread_id)),
        Kind::AnswerInvitation(a) => act(Action::Answer(a.event_id, answer(a.reply)?)),
        Kind::NewEvent(_) => nav(Nav::NewEvent),
        Kind::SaveEvent(s) => act(Action::SaveEvent {
            event: match s.event.ok_or(InvalidIntent)? {
                Saved::EventId(id) => EventToSave::Saved(id),
                Saved::Unsaved(_) => EventToSave::New,
            },
            title: s.title,
            day: s.day_id,
            start: s.start_minute,
            end: s.end_minute,
            calendar: s.calendar_id,
            guests: s.guests,
            place: s.place,
        }),
        Kind::DeleteEvent(d) => act(Action::DeleteEvent(d.event_id)),
        Kind::SetCalendarShown(s) => act(Action::CalendarShown(s.calendar_id, s.shown)),
        Kind::RunAgent(r) => act(Action::Run(r.agent_id)),
        Kind::SetAgentActive(s) => act(Action::AgentActive(s.agent_id, s.active)),
        Kind::SetAgentInstructions(s) => act(Action::AgentInstructions(s.agent_id, s.text)),
        Kind::AddAgentTime(a) => act(Action::AddTime(a.agent_id, a.minute)),
        Kind::RemoveAgentTime(r) => act(Action::RemoveTime(r.agent_id, r.minute)),
        Kind::SetAgentModel(s) => act(Action::AgentModel(s.agent_id, s.model_name)),
        Kind::SetAgentPermission(s) => act(Action::AgentPermission(
            s.agent_id,
            s.permission_id,
            level(s.level)?,
        )),
        Kind::SetAgentAccount(s) => act(Action::AgentAccount(s.agent_id, s.account_id, s.included)),
        Kind::AddNewsTopic(a) => act(Action::AddTopic(a.topic)),
        Kind::RemoveNewsTopic(r) => act(Action::RemoveTopic(r.topic)),
        Kind::RemoveAgent(r) => act(Action::RemoveAgent(r.agent_id)),
        Kind::UpdateBuilder(u) => act(Action::Builder(u.name, u.instructions)),
        Kind::PickIdea(p) => act(Action::Idea(p.idea)),
        Kind::SetBuilderTrigger(s) => act(Action::BuilderTrigger(s.trigger_id, s.chosen)),
        Kind::SetTriggerFilter(s) => act(Action::TriggerFilter(
            s.trigger_id,
            s.filter_id,
            s.choice_id,
        )),
        Kind::SetBuilderSchedule(s) => act(Action::BuilderSchedule(s.choice_id)),
        Kind::SetBuilderInterval(s) => act(Action::BuilderInterval(s.minutes)),
        Kind::SetBuilderAccount(s) => act(Action::BuilderAccount(s.account_id, s.can_see)),
        Kind::SetBuilderPermission(s) => {
            act(Action::BuilderPermission(s.permission_id, level(s.level)?))
        }
        Kind::SetAskBeforeNewRecipient(s) => act(Action::AskNewRecipient(s.ask)),
        Kind::StartDryRun(_) => act(Action::DryRun),
        Kind::CancelDryRun(_) => act(Action::CancelDryRun),
        Kind::CreateAgent(_) => act(Action::CreateAgent),
        Kind::DiscardBuilder(_) => act(Action::DiscardBuilder),
        Kind::SendChatMessage(s) => act(Action::Chat(s.agent_id, s.text)),
        Kind::ClearConversation(c) => act(Action::ClearChat(c.agent_id)),
        Kind::ApproveItem(a) => act(Action::Approve(a.approval_id)),
        Kind::DeclineItem(d) => act(Action::Decline(d.approval_id)),
        Kind::EditApproval(e) => act(Action::EditApproval(e.approval_id)),
        Kind::ReleaseQuarantined(r) => act(Action::Release(r.quarantined_id)),
        Kind::DeleteQuarantined(d) => act(Action::DeleteQuarantined(d.quarantined_id)),
        Kind::AddRule(a) => act(Action::AddRule(
            condition(a.condition)?,
            a.actions
                .into_iter()
                .map(rule_action)
                .collect::<Result<_, _>>()?,
        )),
        Kind::SetRuleEnabled(s) => act(Action::RuleEnabled(s.rule_id, s.enabled)),
        Kind::RemoveRule(r) => act(Action::RemoveRule(r.rule_id)),
        Kind::UnblockSender(u) => act(Action::Unblock(u.address)),
        Kind::SetModelKey(s) => act(Action::SetKey(
            known(ModelProvider::try_from(s.provider))?,
            s.key,
        )),
        Kind::RemoveModelKey(r) => act(Action::RemoveKey(known(ModelProvider::try_from(
            r.provider,
        ))?)),
        Kind::SetTaskModel(s) => act(Action::TaskModel(s.task_id, s.model_name)),
        Kind::SetBudget(s) => act(Action::Budget(match s.budget.ok_or(InvalidIntent)? {
            Budget::Unlimited(_) => None,
            Budget::Daily(typed) => Some(typed),
        })),
        Kind::SetNotifyWhenWaiting(s) => act(Action::Notify(s.notify)),
        Kind::SetAgentsPaused(s) => act(Action::Paused(s.paused)),
        Kind::SetDefaultPermission(s) => {
            act(Action::DefaultPermission(s.permission_id, level(s.level)?))
        }
        Kind::ExportData(_) => act(Action::Export),
        Kind::CancelExport(_) => act(Action::CancelExport),
        Kind::ResetLocalData(_) => act(Action::Reset),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::ui::v1::{
        AnswerInvitation, ContinueWithProvider, SnoozeThread, intent::Kind,
    };

    fn decoded(kind: Kind) -> Option<Action> {
        match input(kind) {
            Ok(Input::Action(action)) => Some(action),
            Ok(_) | Err(_) => None,
        }
    }

    #[test]
    fn unspecified_and_unknown_enum_values_are_decode_errors() {
        let snooze = |preset: SnoozePreset| {
            Kind::SnoozeThread(SnoozeThread {
                thread_id: "t".into(),
                preset: preset as i32,
            })
        };
        assert!(matches!(
            decoded(snooze(SnoozePreset::ThisWeekend)),
            Some(Action::Snooze(t, Snooze::ThisWeekend)) if t == "t"
        ));
        assert!(input(snooze(SnoozePreset::Unspecified)).is_err());
        assert!(
            input(Kind::SnoozeThread(SnoozeThread {
                thread_id: "t".into(),
                preset: 99,
            }))
            .is_err()
        );
        let reply = |reply: i32| {
            Kind::AnswerInvitation(AnswerInvitation {
                event_id: "e".into(),
                reply,
            })
        };
        assert!(matches!(
            decoded(reply(InvitationReply::Maybe as i32)),
            Some(Action::Answer(_, Answer::Tentative))
        ));
        assert!(input(reply(0)).is_err());
        let sign_in = |provider: i32| Kind::ContinueWithProvider(ContinueWithProvider { provider });
        assert!(matches!(
            decoded(sign_in(SignInProvider::Microsoft as i32)),
            Some(Action::SignIn(Identity::Microsoft))
        ));
        assert!(input(sign_in(0)).is_err());
    }

    #[test]
    fn unset_oneofs_and_unspecified_navigation_enums_are_decode_errors() {
        use crate::proto::ui::v1::{
            AddRule, OpenDisclosure, OpenInbox, OpenSettings, SaveEvent, SelectCalendarView,
            SetBudget,
        };
        assert!(input(Kind::OpenInbox(OpenInbox { scope: None })).is_err());
        assert!(
            input(Kind::OpenInbox(OpenInbox {
                scope: Some(InboxScope { scope: None })
            }))
            .is_err()
        );
        assert!(input(Kind::SetBudget(SetBudget { budget: None })).is_err());
        assert!(input(Kind::SaveEvent(SaveEvent::default())).is_err());
        assert!(input(Kind::AddRule(AddRule::default())).is_err());
        assert!(input(Kind::OpenDisclosure(OpenDisclosure { target: None })).is_err());
        let form = |form: Form| {
            Kind::OpenDisclosure(OpenDisclosure {
                target: Some(DisclosureTarget {
                    target: Some(Target::Form(form as i32)),
                }),
            })
        };
        assert!(matches!(
            input(form(Form::NewRule)),
            Ok(Input::Navigation(navigation::Event::Disclose(
                Disclosure::NewRule
            )))
        ));
        assert!(input(form(Form::Search)).is_err(), "not a disclosure");
        assert!(
            input(Kind::OpenSettings(OpenSettings {
                section: SettingsSection::Unspecified as i32
            }))
            .is_err()
        );
        assert!(
            input(Kind::SelectCalendarView(SelectCalendarView {
                mode: CalendarMode::Unspecified as i32
            }))
            .is_err()
        );
        assert!(matches!(
            input(Kind::SelectCalendarView(SelectCalendarView {
                mode: CalendarMode::Year as i32
            })),
            Ok(Input::Navigation(navigation::Event::SelectCalendarMode(
                navigation::Mode::Year
            )))
        ));
    }

    #[test]
    fn retrying_the_connection_is_a_connection_event() {
        assert!(matches!(
            input(Kind::RetryConnection(
                crate::proto::ui::v1::RetryConnection {}
            )),
            Ok(Input::Connection(connection::Event::RetryRequested))
        ));
    }
}
