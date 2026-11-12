-- The sample world. "Now" is Wednesday, November 11 2026, 9:50.
-- Rows may name rows inserted after them; keys are checked at the end of the fill.

PRAGMA defer_foreign_keys = ON;

INSERT INTO accounts VALUES ('personal', 0, 'Personal', 'ali.hassan.dev@gmail.com', 1, 'Gmail', 1, NULL, NULL, 1794390540, 'Ali
sent with EveryMail');
INSERT INTO accounts VALUES ('laurentian', 1, 'Laurentian', 'ali@laurentian.ca', 2, 'Outlook', 1, NULL, NULL, 1794390540, 'Ali
B.Sc. Computer Science, Laurentian University');
INSERT INTO accounts VALUES ('icloud', 2, 'iCloud', 'ali.h@icloud.com', 3, 'iCloud Mail over IMAP', 2, NULL, 'App password expired', 1794390540, NULL);
INSERT INTO folders VALUES ('starred', 0, 'Starred', 1);
INSERT INTO folders VALUES ('sent', 1, 'Sent', 2);
INSERT INTO folders VALUES ('archive', 2, 'Archive', 3);
INSERT INTO folders VALUES ('receipts', 3, 'Receipts', 5);
INSERT INTO folders VALUES ('trash', 4, 'Trash', 6);
INSERT INTO folders VALUES ('quarantine', 5, 'Quarantine', 4);
INSERT INTO threads VALUES ('t-okafor', 'laurentian', 1, 1, 'Prof. Okafor', 'COSC 4456 exam room moved to FA-224', 1794390060, 'Gatekeeper flagged this for you', 2, 'Tomorrow’s parallel computing exam is in FA-224 instead of the usual room. The time is unchanged.', 1, NULL);
INSERT INTO quick_replies VALUES ('t-okafor', 0, 'Thanks for letting us know');
INSERT INTO quick_replies VALUES ('t-okafor', 1, 'Is the exam still at 14:00?');
INSERT INTO quick_replies VALUES ('t-okafor', 2, 'Will office hours be in FA-302 as usual?');
INSERT INTO messages VALUES ('m-okafor', 't-okafor', 0, 0, 'Prof. Okafor', 'Please note the exam will be held in FA-224', 1794390060, 'Hello all,

Please note the exam for COSC 4456 will be held in FA-224. The time and format are unchanged.

Good luck with your studying.', NULL, 1);
INSERT INTO threads VALUES ('t-northline', 'personal', 1, 1, 'Northline Talent', 'Next steps: ML infrastructure interview loop', 1794388320, 'Job application tracker drafted a reply', 1, 'They want to book a three-part loop next week (systems design, coding, team chat) and ask for two time windows.', 1, NULL);
INSERT INTO quick_replies VALUES ('t-northline', 0, 'Tuesday afternoon works best');
INSERT INTO quick_replies VALUES ('t-northline', 1, 'Thursday morning works best');
INSERT INTO quick_replies VALUES ('t-northline', 2, 'Could we do the week after?');
INSERT INTO messages VALUES ('m-northline', 't-northline', 0, 0, 'Dana, Northline Talent', 'Could you send two windows next week that work for you?', 1794388320, 'Hi Ali,

Thanks for the chat last week. We’d love to move you to the full loop for the ML Infrastructure role: a systems design session, a coding session and a conversation with the team.

Could you send two windows next week that work for you?

Best,
Dana', NULL, 0);
INSERT INTO threads VALUES ('t-slides', 'laurentian', 1, 1, 'Mira', 'Slides for Thursday, can you take section 3?', 1794387000, 'Scribe suggested a reply', 2, 'The group agreed on four sections on Monday and Mira booked a rehearsal for tomorrow morning. She now asks you to own section 3 (results) by tonight.', 1, NULL);
INSERT INTO thread_people VALUES ('t-slides', 0, 'Priya');
INSERT INTO thread_people VALUES ('t-slides', 1, 'Tomas');
INSERT INTO quick_replies VALUES ('t-slides', 0, 'Yes, I’ll have section 3 in the deck tonight');
INSERT INTO quick_replies VALUES ('t-slides', 1, 'Can I take it but need until tomorrow 8:00?');
INSERT INTO quick_replies VALUES ('t-slides', 2, 'I can’t this week, could someone swap?');
INSERT INTO messages VALUES ('m-slides-1', 't-slides', 0, 0, 'Mira', 'Kicking off the deck', 1794240720, 'Kicking off the deck for Thursday. I made a shared file with four sections: problem, method, results, next steps. Claim one when you can!', NULL, 0);
INSERT INTO attachments VALUES ('f-deck-outline', 'm-slides-1', 0, 'thursday-deck-outline.pptx', 1887436, 5);
INSERT INTO messages VALUES ('m-slides-2', 't-slides', 1, 1, 'Ali', 'Happy to help where needed', 1794249000, 'Happy to help where needed. I have the benchmark numbers if anyone wants them.', NULL, 0);
INSERT INTO messages VALUES ('m-slides-3', 't-slides', 2, 0, 'Mira', 'Invitation: Deck rehearsal', 1794309600, 'Booked FA-110 so we can run through it once together.', 'e-rehearsal', 0);
INSERT INTO attachments VALUES ('f-rehearsal-invite', 'm-slides-3', 0, 'deck-rehearsal.ics', 2048, 7);
INSERT INTO messages VALUES ('m-slides-4', 't-slides', 3, 0, 'Mira', 'Could you take section 3, the results?', 1794387000, 'Hey! I split the deck into four parts. Could you take section 3, the results? Would be great to have it by tonight so we can rehearse tomorrow morning.', NULL, 0);
INSERT INTO threads VALUES ('t-harbour', 'personal', 1, 1, 'Harbour Labs', 'We received your application', 1794386460, 'Job application tracker logged this to Applications', 3, 'Harbour Labs confirms your application for the Platform Engineer role and will answer within two weeks.', 0, NULL);
INSERT INTO quick_replies VALUES ('t-harbour', 0, 'Thanks, looking forward to it');
INSERT INTO quick_replies VALUES ('t-harbour', 1, 'Happy to share more work samples');
INSERT INTO quick_replies VALUES ('t-harbour', 2, 'Could you share the team’s stack?');
INSERT INTO messages VALUES ('m-harbour', 't-harbour', 0, 0, 'Harbour Labs', 'Thanks for applying to the Platform Engineer role', 1794386460, 'Hi Ali,

Thanks for applying to the Platform Engineer role at Harbour Labs. Our team will review your application and get back to you within two weeks.

Harbour Labs recruiting', NULL, 0);
INSERT INTO threads VALUES ('t-ridgeway', 'personal', 1, 2, 'Ridgeway Hosting', 'Invoice #4471 for November', 1794385800, 'Receipts clerk filed this', 3, 'Monthly hosting invoice. Already filed to Receipts and logged in your budget sheet.', 0, NULL);
INSERT INTO thread_folders VALUES ('t-ridgeway', 'receipts');
INSERT INTO quick_replies VALUES ('t-ridgeway', 0, 'Thanks, received');
INSERT INTO quick_replies VALUES ('t-ridgeway', 1, 'Could you send it as a PDF?');
INSERT INTO quick_replies VALUES ('t-ridgeway', 2, 'Please update my billing address');
INSERT INTO messages VALUES ('m-ridgeway', 't-ridgeway', 0, 0, 'Ridgeway Hosting', 'Your invoice for November is ready', 1794385800, 'Your invoice for November is ready. It will be charged to the card on file.', NULL, 0);
INSERT INTO attachments VALUES ('f-invoice-4471', 'm-ridgeway', 0, 'invoice-4471.pdf', 88064, 2);
INSERT INTO threads VALUES ('t-campus', 'laurentian', 1, 2, 'Campus Rec', 'Intramural sign-ups close Friday', 1794384120, 'Gatekeeper sorted this into Updates', 3, 'Winter intramural registration closes Friday at 17:00.', 0, NULL);
INSERT INTO quick_replies VALUES ('t-campus', 0, 'Sign me up for climbing');
INSERT INTO quick_replies VALUES ('t-campus', 1, 'Is there a waitlist?');
INSERT INTO quick_replies VALUES ('t-campus', 2, 'Thanks for the reminder');
INSERT INTO messages VALUES ('m-campus', 't-campus', 0, 0, 'Campus Rec', 'Registration closes this Friday', 1794384120, 'Registration for winter intramurals closes this Friday at 17:00.', NULL, 0);
INSERT INTO attachments VALUES ('f-intramural-poster', 'm-campus', 0, 'intramural-poster.png', 655360, 1);
INSERT INTO threads VALUES ('t-pragmatic-digest', 'icloud', 1, 3, 'The Pragmatic Digest', 'This week: queues, caches and backpressure', 1794380700, 'Gatekeeper sorted this into Reading · in the 7:30 briefing', 3, NULL, 0, NULL);
INSERT INTO quick_replies VALUES ('t-pragmatic-digest', 0, 'Thanks');
INSERT INTO quick_replies VALUES ('t-pragmatic-digest', 1, 'Unsubscribe me');
INSERT INTO quick_replies VALUES ('t-pragmatic-digest', 2, 'More like this');
INSERT INTO messages VALUES ('m-pragmatic-digest', 't-pragmatic-digest', 0, 0, 'The Pragmatic Digest', 'Queues, caches and backpressure', 1794380700, 'This week we look at queues, caches and how backpressure keeps systems upright.', NULL, 0);
INSERT INTO threads VALUES ('t-parcel', 'personal', 1, 2, 'Parcelway', 'Your package is out for delivery', 1794312000, 'Gatekeeper sorted this into Updates', 3, NULL, 0, NULL);
INSERT INTO quick_replies VALUES ('t-parcel', 0, 'Please leave it at the door');
INSERT INTO quick_replies VALUES ('t-parcel', 1, 'Can I pick it up instead?');
INSERT INTO quick_replies VALUES ('t-parcel', 2, 'Thanks');
INSERT INTO messages VALUES ('m-parcel', 't-parcel', 0, 0, 'Parcelway', 'Your package is out for delivery', 1794312000, 'Your package is out for delivery and should arrive today before 20:00.', NULL, 0);
INSERT INTO threads VALUES ('t-dev-weekly', 'icloud', 1, 3, 'Dev Weekly', 'Zig 0.15 notes, async I/O revisited', 1794312000, 'Gatekeeper sorted this into Reading · in the Tue 18:00 briefing', 3, NULL, 0, NULL);
INSERT INTO quick_replies VALUES ('t-dev-weekly', 0, 'Thanks');
INSERT INTO quick_replies VALUES ('t-dev-weekly', 1, 'Unsubscribe me');
INSERT INTO quick_replies VALUES ('t-dev-weekly', 2, 'More like this');
INSERT INTO messages VALUES ('m-dev-weekly', 't-dev-weekly', 0, 0, 'Dev Weekly', 'Release notes round-up', 1794312000, 'Release notes round-up and a long read on async I/O.', NULL, 0);
INSERT INTO threads VALUES ('t-ostra', 'personal', 2, 1, 'Ostra Robotics', 'Application: Platform Engineer', 1794052800, 'Follow-up nudger drafted a nudge', 1, 'You applied to Ostra Robotics on Saturday and nobody has answered yet.', 0, NULL);
INSERT INTO quick_replies VALUES ('t-ostra', 0, 'Just checking in');
INSERT INTO quick_replies VALUES ('t-ostra', 1, 'Happy to share more');
INSERT INTO quick_replies VALUES ('t-ostra', 2, 'Is the role still open?');
INSERT INTO messages VALUES ('m-ostra', 't-ostra', 0, 1, 'Ali', 'My application for the Platform Engineer role', 1794046500, 'Hi Sam,

Please find my application for the Platform Engineer role attached. I’d be glad to talk about how I could help the team.

Best,
Ali', NULL, 0);
INSERT INTO threads VALUES ('t-lunch', 'icloud', 2, 1, 'Mira', 'Lunch Monday?', 1794139200, 'Gatekeeper sorted this into For you', 3, 'Mira suggests lunch at the student centre on Monday at 12:00.', 0, NULL);
INSERT INTO quick_replies VALUES ('t-lunch', 0, 'See you there');
INSERT INTO quick_replies VALUES ('t-lunch', 1, 'Can we do 12:30?');
INSERT INTO quick_replies VALUES ('t-lunch', 2, 'Next week instead?');
INSERT INTO messages VALUES ('m-lunch', 't-lunch', 0, 0, 'Mira', 'Lunch on Monday at the student centre?', 1794164640, 'Lunch on Monday at the student centre? 12:00 works for me.', NULL, 0);
INSERT INTO calendars VALUES ('cal-personal', 0, 'Personal', 1, 'Google Calendar', 1, 1);
INSERT INTO calendars VALUES ('cal-laurentian', 1, 'Laurentian', 2, 'Outlook calendar', 2, 1);
INSERT INTO calendars VALUES ('cal-icloud', 2, 'iCloud', 3, 'iCloud through CalDAV', 3, 1);
INSERT INTO events VALUES ('e-advising', 'cal-laurentian', '2026-11-03', 840, 870, 'Course advising', 'Student services');
INSERT INTO events VALUES ('e-project-retro', 'cal-laurentian', '2026-11-16', 780, 840, 'Project retro', 'FA-110');
INSERT INTO events VALUES ('e-climbing-next', 'cal-icloud', '2026-11-20', 1080, 1170, 'Climbing with Jordan', 'Boulder gym');
INSERT INTO events VALUES ('e-final-project', 'cal-laurentian', '2026-12-01', 1380, 1410, 'COSC 4456 final project due', 'Course site');
INSERT INTO events VALUES ('e-lecture-mon', 'cal-laurentian', '2026-11-09', 600, 690, 'COSC 4456 lecture', 'FA-110');
INSERT INTO events VALUES ('e-lunch', 'cal-icloud', '2026-11-09', 720, 780, 'Lunch with Mira', 'Student centre');
INSERT INTO event_guests VALUES ('e-lunch', 0, 'Mira');
INSERT INTO events VALUES ('e-lecture-wed', 'cal-laurentian', '2026-11-11', 600, 690, 'COSC 4456 lecture', 'FA-110');
INSERT INTO events VALUES ('e-slides', 'cal-personal', '2026-11-11', 1140, 1230, 'Section 3 slides', 'Focus time');
INSERT INTO events VALUES ('e-rehearsal', 'cal-laurentian', '2026-11-12', 540, 600, 'Deck rehearsal', 'FA-110');
INSERT INTO event_guests VALUES ('e-rehearsal', 0, 'Mira');
INSERT INTO event_guests VALUES ('e-rehearsal', 1, 'Priya');
INSERT INTO event_guests VALUES ('e-rehearsal', 2, 'Tomas');
INSERT INTO invitations VALUES ('e-rehearsal', 'Mira', 'Mira, you and 2 others', 1);
INSERT INTO events VALUES ('e-office-hours', 'cal-laurentian', '2026-11-12', 600, 660, 'Office hours, Prof. Okafor', 'FA-302');
INSERT INTO events VALUES ('e-exam', 'cal-laurentian', '2026-11-12', 840, 960, 'COSC 4456 exam', 'FA-224');
INSERT INTO events VALUES ('e-brightwave', 'cal-personal', '2026-11-13', 600, 690, 'Brightwave AI assessment', 'Online, 90 min');
INSERT INTO events VALUES ('e-intramural', 'cal-laurentian', '2026-11-13', 1020, 1050, 'Intramural sign-ups close', 'Campus Rec');
INSERT INTO events VALUES ('e-climbing', 'cal-icloud', '2026-11-14', 660, 750, 'Climbing with Jordan', 'Boulder gym');
INSERT INTO event_guests VALUES ('e-climbing', 0, 'Jordan');
INSERT INTO events VALUES ('e-call-home', 'cal-icloud', '2026-11-15', 1080, 1140, 'Call home', '');
INSERT INTO actions VALUES ('sort-categories', 0, 'Sort into categories', NULL, 0);
INSERT INTO actions VALUES ('flag', 1, 'Flag', NULL, 0);
INSERT INTO actions VALUES ('quarantine', 2, 'Quarantine', NULL, 0);
INSERT INTO actions VALUES ('block-sender', 3, 'Block sender', NULL, 0);
INSERT INTO actions VALUES ('report-spam', 4, 'Report spam', NULL, 0);
INSERT INTO actions VALUES ('notify', 5, 'Notify you', NULL, 0);
INSERT INTO actions VALUES ('write-summaries', 6, 'Write summaries', NULL, 0);
INSERT INTO actions VALUES ('suggest-replies', 7, 'Suggest replies', NULL, 0);
INSERT INTO actions VALUES ('write-briefing', 8, 'Write the briefing', NULL, 0);
INSERT INTO actions VALUES ('mail-briefing', 9, 'Mail the briefing to you', NULL, 0);
INSERT INTO actions VALUES ('read-search', 10, 'Read and search mail and calendars', NULL, 0);
INSERT INTO actions VALUES ('read-sent', 11, 'Read sent mail', NULL, 0);
INSERT INTO actions VALUES ('read-mail', 12, 'Read mail', NULL, 0);
INSERT INTO actions VALUES ('read-label', 13, 'Read and label', 'Sort, tag and summarise mail', 1);
INSERT INTO actions VALUES ('update-sheets', 14, 'Update sheets', 'Add rows to sheets you name', 1);
INSERT INTO actions VALUES ('update-applications-sheet', 15, 'Update the Applications sheet', NULL, 0);
INSERT INTO actions VALUES ('update-budget-sheet', 16, 'Update the budget sheet', NULL, 0);
INSERT INTO actions VALUES ('calendar', 17, 'Calendar', 'Read free time, add events', 1);
INSERT INTO actions VALUES ('add-events', 18, 'Add calendar events', NULL, 0);
INSERT INTO actions VALUES ('move-folders', 19, 'Move to your folders', NULL, 0);
INSERT INTO actions VALUES ('draft-replies', 20, 'Draft replies', 'Saved to Drafts in your voice', 1);
INSERT INTO actions VALUES ('reply-forward', 21, 'Reply or forward', NULL, 0);
INSERT INTO actions VALUES ('send', 22, 'Send', 'Mail leaves your account', 1);
INSERT INTO actions VALUES ('archive', 23, 'Archive', 'Clear handled mail', 1);
INSERT INTO actions VALUES ('unsubscribe', 24, 'Unsubscribe', NULL, 0);
INSERT INTO actions VALUES ('delete', 25, 'Delete', 'Moves to trash', 1);
INSERT INTO agents VALUES ('gatekeeper', 0, 'Gatekeeper', 'Sort every new mail into For you, Updates, Reading or one of your folders, and flag what needs you.
Check bulk headers, the provider’s spam verdict, SPF, DKIM and DMARC, the block list and whether you have written to the sender.
Hand likely scams to Security Guard.
When an account is connected, sort its last 90 days and ask once per bulk sender whether to unsubscribe.', 1, 'Anything from my project group goes to For you.', 'Claude Sonnet', 1, 2, 'On every new mail, after your rules', NULL, NULL, 1794390600, 'Without a model key it runs its built-in checks only', 1, 0);
INSERT INTO agent_permissions VALUES ('gatekeeper', 'sort-categories', 3, NULL);
INSERT INTO agent_permissions VALUES ('gatekeeper', 'flag', 3, NULL);
INSERT INTO agent_permissions VALUES ('gatekeeper', 'move-folders', 3, NULL);
INSERT INTO agent_permissions VALUES ('gatekeeper', 'unsubscribe', 2, NULL);
INSERT INTO agent_accounts VALUES ('gatekeeper', 'personal');
INSERT INTO agent_accounts VALUES ('gatekeeper', 'laurentian');
INSERT INTO agent_accounts VALUES ('gatekeeper', 'icloud');
INSERT INTO agent_stats VALUES ('gatekeeper', 0, '8', 'sorted today');
INSERT INTO agent_stats VALUES ('gatekeeper', 1, '2', 'waiting for you');
INSERT INTO agent_stats VALUES ('gatekeeper', 2, '9:41', 'last run');
INSERT INTO result_stats VALUES ('gatekeeper', 0, '4', 'For you');
INSERT INTO result_stats VALUES ('gatekeeper', 1, '2', 'Updates');
INSERT INTO result_stats VALUES ('gatekeeper', 2, '1', 'Reading');
INSERT INTO result_stats VALUES ('gatekeeper', 3, '1', 'to Security Guard');
INSERT INTO result_items VALUES ('ri-gatekeeper-0-0', 'gatekeeper', 'Sorted today', 0, 0, 1794390060, 'Flagged for you', 'A Laurentian address you answer often, about your exam');
INSERT INTO item_refs VALUES ('ri-gatekeeper-0-0', 0, 1, 't-okafor');
INSERT INTO result_items VALUES ('ri-gatekeeper-0-1', 'gatekeeper', 'Sorted today', 0, 1, 1794388320, 'Sorted into For you', 'A recruiter answering your application');
INSERT INTO item_refs VALUES ('ri-gatekeeper-0-1', 0, 1, 't-northline');
INSERT INTO result_items VALUES ('ri-gatekeeper-0-2', 'gatekeeper', 'Sorted today', 0, 2, 1794387000, 'Sorted into For you', 'From your project group, as your instructions say');
INSERT INTO item_refs VALUES ('ri-gatekeeper-0-2', 0, 1, 't-slides');
INSERT INTO result_items VALUES ('ri-gatekeeper-0-3', 'gatekeeper', 'Sorted today', 0, 3, 1794386460, 'Sorted into For you', 'Confirms an application you sent');
INSERT INTO item_refs VALUES ('ri-gatekeeper-0-3', 0, 1, 't-harbour');
INSERT INTO result_items VALUES ('ri-gatekeeper-0-4', 'gatekeeper', 'Sorted today', 0, 4, 1794385800, 'Sorted into Updates', 'An invoice: your rule hands it to Receipts clerk');
INSERT INTO item_refs VALUES ('ri-gatekeeper-0-4', 0, 1, 't-ridgeway');
INSERT INTO result_items VALUES ('ri-gatekeeper-0-5', 'gatekeeper', 'Sorted today', 0, 5, 1794384120, 'Sorted into Updates', 'A campus mailing list; nothing to answer');
INSERT INTO item_refs VALUES ('ri-gatekeeper-0-5', 0, 1, 't-campus');
INSERT INTO result_items VALUES ('ri-gatekeeper-0-6', 'gatekeeper', 'Sorted today', 0, 6, 1794381120, 'Handed to Security Guard', 'Failed DMARC and pretends to be your mail provider');
INSERT INTO item_refs VALUES ('ri-gatekeeper-0-6', 0, 4, 'q-mailbox');
INSERT INTO result_items VALUES ('ri-gatekeeper-0-7', 'gatekeeper', 'Sorted today', 0, 7, 1794380700, 'Sorted into Reading', 'A newsletter, condensed in the briefing');
INSERT INTO item_refs VALUES ('ri-gatekeeper-0-7', 0, 1, 't-pragmatic-digest');
INSERT INTO result_items VALUES ('ri-gatekeeper-1-0', 'gatekeeper', 'Asks you', 1, 0, 1794225600, 'Unsubscribe from Jobstream weekly?', '46 mails in 90 days, you opened 2');
INSERT INTO item_refs VALUES ('ri-gatekeeper-1-0', 0, 3, 'a-jobstream');
INSERT INTO result_items VALUES ('ri-gatekeeper-1-1', 'gatekeeper', 'Asks you', 1, 1, 1794225600, 'Unsubscribe from Campus Store deals?', '31 mails in 90 days, you opened none');
INSERT INTO item_refs VALUES ('ri-gatekeeper-1-1', 0, 3, 'a-campus-store');
INSERT INTO runs VALUES ('r-gatekeeper-8', 'gatekeeper', 1794390060, 1, NULL, 2, 1240, 1, NULL, 'Flagged 1');
INSERT INTO cause_refs VALUES ('r-gatekeeper-8', 0, 1, 't-okafor');
INSERT INTO run_steps VALUES ('r-gatekeeper-8', 0, 'Read', 'Prof. Okafor’s mail and your past replies', 1);
INSERT INTO step_refs VALUES ('r-gatekeeper-8', 0, 0, 1, 't-okafor');
INSERT INTO run_steps VALUES ('r-gatekeeper-8', 1, 'Sorted', 'Into For you', 2);
INSERT INTO step_refs VALUES ('r-gatekeeper-8', 1, 0, 1, 't-okafor');
INSERT INTO run_steps VALUES ('r-gatekeeper-8', 2, 'Flagged', 'As needing you', 2);
INSERT INTO step_refs VALUES ('r-gatekeeper-8', 2, 0, 1, 't-okafor');
INSERT INTO runs VALUES ('r-gatekeeper-7', 'gatekeeper', 1794388320, 1, NULL, 1, 980, 1, NULL, 'Sorted 1 into For you');
INSERT INTO cause_refs VALUES ('r-gatekeeper-7', 0, 1, 't-northline');
INSERT INTO run_steps VALUES ('r-gatekeeper-7', 0, 'Read', 'Northline Talent’s reply', 1);
INSERT INTO step_refs VALUES ('r-gatekeeper-7', 0, 0, 1, 't-northline');
INSERT INTO run_steps VALUES ('r-gatekeeper-7', 1, 'Sorted', 'Into For you', 2);
INSERT INTO step_refs VALUES ('r-gatekeeper-7', 1, 0, 1, 't-northline');
INSERT INTO runs VALUES ('r-gatekeeper-6', 'gatekeeper', 1794387000, 1, NULL, 1, 1050, 1, NULL, 'Sorted 1 into For you');
INSERT INTO cause_refs VALUES ('r-gatekeeper-6', 0, 1, 't-slides');
INSERT INTO run_steps VALUES ('r-gatekeeper-6', 0, 'Read', 'Mira’s mail to the project group', 1);
INSERT INTO step_refs VALUES ('r-gatekeeper-6', 0, 0, 1, 't-slides');
INSERT INTO run_steps VALUES ('r-gatekeeper-6', 1, 'Sorted', 'Into For you, by your instructions', 2);
INSERT INTO step_refs VALUES ('r-gatekeeper-6', 1, 0, 1, 't-slides');
INSERT INTO runs VALUES ('r-gatekeeper-5', 'gatekeeper', 1794386460, 1, NULL, 1, 860, 1, NULL, 'Sorted 1 into For you');
INSERT INTO cause_refs VALUES ('r-gatekeeper-5', 0, 1, 't-harbour');
INSERT INTO run_steps VALUES ('r-gatekeeper-5', 0, 'Read', 'Harbour Labs’ confirmation', 1);
INSERT INTO step_refs VALUES ('r-gatekeeper-5', 0, 0, 1, 't-harbour');
INSERT INTO run_steps VALUES ('r-gatekeeper-5', 1, 'Sorted', 'Into For you', 2);
INSERT INTO step_refs VALUES ('r-gatekeeper-5', 1, 0, 1, 't-harbour');
INSERT INTO runs VALUES ('r-gatekeeper-4', 'gatekeeper', 1794385800, 1, NULL, 1, 0, 1, NULL, 'Sorted 1 by your rule');
INSERT INTO cause_refs VALUES ('r-gatekeeper-4', 0, 1, 't-ridgeway');
INSERT INTO run_steps VALUES ('r-gatekeeper-4', 0, 'Sorted', 'Into Updates', 2);
INSERT INTO step_refs VALUES ('r-gatekeeper-4', 0, 0, 1, 't-ridgeway');
INSERT INTO run_steps VALUES ('r-gatekeeper-4', 1, 'Handed', 'To Receipts clerk, by your rule', 2);
INSERT INTO step_refs VALUES ('r-gatekeeper-4', 1, 0, 1, 't-ridgeway');
INSERT INTO runs VALUES ('r-gatekeeper-3', 'gatekeeper', 1794381120, 1, NULL, 2, 1310, 1, NULL, 'Handed 1 to Security Guard');
INSERT INTO cause_refs VALUES ('r-gatekeeper-3', 0, 4, 'q-mailbox');
INSERT INTO run_steps VALUES ('r-gatekeeper-3', 0, 'Read', 'SPF, DKIM and DMARC: DMARC failed', 1);
INSERT INTO step_refs VALUES ('r-gatekeeper-3', 0, 0, 4, 'q-mailbox');
INSERT INTO run_steps VALUES ('r-gatekeeper-3', 1, 'Handed', 'To Security Guard as a likely scam', 2);
INSERT INTO step_refs VALUES ('r-gatekeeper-3', 1, 0, 4, 'q-mailbox');
INSERT INTO runs VALUES ('r-gatekeeper-2', 'gatekeeper', 1794380700, 1, NULL, 1, 0, 1, NULL, 'Sorted 1 into Reading');
INSERT INTO cause_refs VALUES ('r-gatekeeper-2', 0, 1, 't-pragmatic-digest');
INSERT INTO run_steps VALUES ('r-gatekeeper-2', 0, 'Sorted', 'Into Reading, from its bulk headers', 2);
INSERT INTO step_refs VALUES ('r-gatekeeper-2', 0, 0, 1, 't-pragmatic-digest');
INSERT INTO runs VALUES ('r-gatekeeper-1', 'gatekeeper', 1794234000, 6, 'Laurentian', 38, 0, 2, NULL, 'Swept 90 days of Laurentian · 2 questions');
INSERT INTO run_steps VALUES ('r-gatekeeper-1', 0, 'Sorted', '2,140 mails from the last 90 days', 2);
INSERT INTO run_steps VALUES ('r-gatekeeper-1', 1, 'Unsubscribe', 'From Jobstream weekly', 3);
INSERT INTO step_refs VALUES ('r-gatekeeper-1', 1, 0, 3, 'a-jobstream');
INSERT INTO run_steps VALUES ('r-gatekeeper-1', 2, 'Unsubscribe', 'From Campus Store deals', 3);
INSERT INTO step_refs VALUES ('r-gatekeeper-1', 2, 0, 3, 'a-campus-store');
INSERT INTO agents VALUES ('security-guard', 1, 'Security Guard', 'Move scam and malicious mail to Quarantine.
Block the sender and report the mail to the provider as spam.
Tell me what it caught.
Never follow a link. Never unsubscribe.', 1, '', 'Claude Sonnet', 1, 2, 'When Gatekeeper flags mail as a scam or malicious', NULL, NULL, 1794390600, 'Without a model key it runs its built-in checks only', 1, 0);
INSERT INTO agent_permissions VALUES ('security-guard', 'quarantine', 3, NULL);
INSERT INTO agent_permissions VALUES ('security-guard', 'block-sender', 3, NULL);
INSERT INTO agent_permissions VALUES ('security-guard', 'report-spam', 3, NULL);
INSERT INTO agent_permissions VALUES ('security-guard', 'notify', 3, NULL);
INSERT INTO agent_accounts VALUES ('security-guard', 'personal');
INSERT INTO agent_accounts VALUES ('security-guard', 'laurentian');
INSERT INTO agent_accounts VALUES ('security-guard', 'icloud');
INSERT INTO agent_stats VALUES ('security-guard', 0, '3', 'in Quarantine');
INSERT INTO agent_stats VALUES ('security-guard', 1, '3', 'senders blocked');
INSERT INTO agent_stats VALUES ('security-guard', 2, '7:12', 'last catch');
INSERT INTO result_stats VALUES ('security-guard', 0, '3', 'in Quarantine');
INSERT INTO result_stats VALUES ('security-guard', 1, '3', 'senders blocked');
INSERT INTO result_items VALUES ('ri-security-guard-0-0', 'security-guard', 'Caught', 0, 0, 1794381120, 'Quarantined “Your mailbox will be closed today”', 'Pretended to be your mail provider. Sender blocked and reported to Gmail');
INSERT INTO item_refs VALUES ('ri-security-guard-0-0', 0, 4, 'q-mailbox');
INSERT INTO result_items VALUES ('ri-security-guard-0-1', 'security-guard', 'Caught', 0, 1, 1794312000, 'Quarantined “Customs fee unpaid for your parcel”', 'Asked for a fee through a link. Sender blocked and reported to Outlook');
INSERT INTO item_refs VALUES ('ri-security-guard-0-1', 0, 4, 'q-customs');
INSERT INTO result_items VALUES ('ri-security-guard-0-2', 'security-guard', 'Caught', 0, 2, 1793707200, 'Quarantined “Final hours: 80% off everything”', 'A lookalike shop domain. Sender blocked');
INSERT INTO item_refs VALUES ('ri-security-guard-0-2', 0, 4, 'q-deals');
INSERT INTO runs VALUES ('r-security-guard-3', 'security-guard', 1794381120, 5, 'Gatekeeper', 2, 1580, 1, NULL, 'Quarantined 1 · sender blocked and reported');
INSERT INTO cause_refs VALUES ('r-security-guard-3', 0, 4, 'q-mailbox');
INSERT INTO run_steps VALUES ('r-security-guard-3', 0, 'Read', 'Headers and links, without opening them', 1);
INSERT INTO step_refs VALUES ('r-security-guard-3', 0, 0, 4, 'q-mailbox');
INSERT INTO run_steps VALUES ('r-security-guard-3', 1, 'Quarantined', '“Your mailbox will be closed today”', 2);
INSERT INTO step_refs VALUES ('r-security-guard-3', 1, 0, 4, 'q-mailbox');
INSERT INTO run_steps VALUES ('r-security-guard-3', 2, 'Blocked', 'security@acc0unt-verify.net', 2);
INSERT INTO run_steps VALUES ('r-security-guard-3', 3, 'Reported', 'To Gmail as spam', 2);
INSERT INTO run_steps VALUES ('r-security-guard-3', 4, 'Notified', 'You, in the briefing', 2);
INSERT INTO runs VALUES ('r-security-guard-2', 'security-guard', 1794329280, 5, 'Gatekeeper', 2, 1420, 1, NULL, 'Quarantined 1 · sender blocked and reported');
INSERT INTO cause_refs VALUES ('r-security-guard-2', 0, 4, 'q-customs');
INSERT INTO run_steps VALUES ('r-security-guard-2', 0, 'Read', 'Headers and links, without opening them', 1);
INSERT INTO step_refs VALUES ('r-security-guard-2', 0, 0, 4, 'q-customs');
INSERT INTO run_steps VALUES ('r-security-guard-2', 1, 'Quarantined', '“Customs fee unpaid for your parcel”', 2);
INSERT INTO step_refs VALUES ('r-security-guard-2', 1, 0, 4, 'q-customs');
INSERT INTO run_steps VALUES ('r-security-guard-2', 2, 'Blocked', 'noreply@shipping-hold.info', 2);
INSERT INTO run_steps VALUES ('r-security-guard-2', 3, 'Reported', 'To Outlook as spam', 2);
INSERT INTO runs VALUES ('r-security-guard-1', 'security-guard', 1793704800, 5, 'Gatekeeper', 1, 1190, 1, NULL, 'Quarantined 1 · sender blocked');
INSERT INTO cause_refs VALUES ('r-security-guard-1', 0, 4, 'q-deals');
INSERT INTO run_steps VALUES ('r-security-guard-1', 0, 'Read', 'Sender domain and links', 1);
INSERT INTO step_refs VALUES ('r-security-guard-1', 0, 0, 4, 'q-deals');
INSERT INTO run_steps VALUES ('r-security-guard-1', 1, 'Quarantined', '“Final hours: 80% off everything”', 2);
INSERT INTO step_refs VALUES ('r-security-guard-1', 1, 0, 4, 'q-deals');
INSERT INTO run_steps VALUES ('r-security-guard-1', 2, 'Blocked', 'deals@megasavings.shop', 2);
INSERT INTO agents VALUES ('scribe', 2, 'Scribe', 'Write a summary of a thread the first time it is needed.
Suggest short replies in your voice.', 1, 'Keep summaries to two sentences.', 'Gemini Pro', 3, 2, 'The first time you open a thread or another agent needs it, never on arrival', NULL, NULL, 1794390600, 'Off until you add a model key', 1, 0);
INSERT INTO agent_permissions VALUES ('scribe', 'write-summaries', 3, NULL);
INSERT INTO agent_permissions VALUES ('scribe', 'suggest-replies', 3, NULL);
INSERT INTO agent_accounts VALUES ('scribe', 'personal');
INSERT INTO agent_accounts VALUES ('scribe', 'laurentian');
INSERT INTO agent_accounts VALUES ('scribe', 'icloud');
INSERT INTO agent_stats VALUES ('scribe', 0, '63', 'summaries this week');
INSERT INTO agent_stats VALUES ('scribe', 1, '18', 'replies suggested');
INSERT INTO agent_stats VALUES ('scribe', 2, '9:44', 'last run');
INSERT INTO result_items VALUES ('ri-scribe-0-0', 'scribe', 'Summaries', 0, 0, 1794390240, 'Northline Talent’s interview loop', 'Dana wants two time windows next week for a three-part loop: systems design, coding and a team chat.');
INSERT INTO item_refs VALUES ('ri-scribe-0-0', 0, 1, 't-northline');
INSERT INTO result_items VALUES ('ri-scribe-0-1', 'scribe', 'Summaries', 0, 1, 1794390120, 'Exam room change', 'Thursday’s COSC 4456 exam moves from FA-110 to FA-224, same time.');
INSERT INTO item_refs VALUES ('ri-scribe-0-1', 0, 1, 't-okafor');
INSERT INTO result_items VALUES ('ri-scribe-0-2', 'scribe', 'Summaries', 0, 2, 1794387120, 'Slides for Thursday', 'The group split the deck four ways on Monday. Mira asks you to own section 3, the results, by tonight.');
INSERT INTO item_refs VALUES ('ri-scribe-0-2', 0, 1, 't-slides');
INSERT INTO result_items VALUES ('ri-scribe-1-0', 'scribe', 'Suggested replies', 1, 0, 1794387120, '3 replies for Mira', '“Yes, I’ll have section 3 in the deck tonight” and 2 more');
INSERT INTO item_refs VALUES ('ri-scribe-1-0', 0, 1, 't-slides');
INSERT INTO result_items VALUES ('ri-scribe-1-1', 'scribe', 'Suggested replies', 1, 1, 1794390240, '3 replies for Northline Talent', '“Tuesday afternoon works best” and 2 more');
INSERT INTO item_refs VALUES ('ri-scribe-1-1', 0, 1, 't-northline');
INSERT INTO runs VALUES ('r-scribe-5', 'scribe', 1794390240, 5, 'Job application tracker', 3, 2210, 1, NULL, 'Summarized 1 · 3 replies');
INSERT INTO cause_refs VALUES ('r-scribe-5', 0, 1, 't-northline');
INSERT INTO run_steps VALUES ('r-scribe-5', 0, 'Read', '2 messages', 1);
INSERT INTO step_refs VALUES ('r-scribe-5', 0, 0, 1, 't-northline');
INSERT INTO run_steps VALUES ('r-scribe-5', 1, 'Wrote', 'A two-sentence summary', 2);
INSERT INTO step_refs VALUES ('r-scribe-5', 1, 0, 1, 't-northline');
INSERT INTO run_steps VALUES ('r-scribe-5', 2, 'Suggested', '3 quick replies', 2);
INSERT INTO step_refs VALUES ('r-scribe-5', 2, 0, 1, 't-northline');
INSERT INTO runs VALUES ('r-scribe-4', 'scribe', 1794390120, 2, NULL, 2, 1630, 1, NULL, 'Summarized 1');
INSERT INTO cause_refs VALUES ('r-scribe-4', 0, 1, 't-okafor');
INSERT INTO run_steps VALUES ('r-scribe-4', 0, 'Read', '1 message', 1);
INSERT INTO step_refs VALUES ('r-scribe-4', 0, 0, 1, 't-okafor');
INSERT INTO run_steps VALUES ('r-scribe-4', 1, 'Wrote', 'A two-sentence summary', 2);
INSERT INTO step_refs VALUES ('r-scribe-4', 1, 0, 1, 't-okafor');
INSERT INTO runs VALUES ('r-scribe-3', 'scribe', 1794387120, 2, NULL, 4, 3480, 1, NULL, 'Summarized Mira’s thread · 3 replies');
INSERT INTO cause_refs VALUES ('r-scribe-3', 0, 1, 't-slides');
INSERT INTO run_steps VALUES ('r-scribe-3', 0, 'Read', '4 messages', 1);
INSERT INTO step_refs VALUES ('r-scribe-3', 0, 0, 1, 't-slides');
INSERT INTO run_steps VALUES ('r-scribe-3', 1, 'Wrote', 'A two-sentence summary', 2);
INSERT INTO step_refs VALUES ('r-scribe-3', 1, 0, 1, 't-slides');
INSERT INTO run_steps VALUES ('r-scribe-3', 2, 'Suggested', '3 quick replies', 2);
INSERT INTO step_refs VALUES ('r-scribe-3', 2, 0, 1, 't-slides');
INSERT INTO runs VALUES ('r-scribe-2', 'scribe', 1794386700, 2, NULL, 2, 1290, 1, NULL, 'Summarized 1');
INSERT INTO cause_refs VALUES ('r-scribe-2', 0, 1, 't-harbour');
INSERT INTO run_steps VALUES ('r-scribe-2', 0, 'Read', '1 message', 1);
INSERT INTO step_refs VALUES ('r-scribe-2', 0, 0, 1, 't-harbour');
INSERT INTO run_steps VALUES ('r-scribe-2', 1, 'Wrote', 'A two-sentence summary', 2);
INSERT INTO step_refs VALUES ('r-scribe-2', 1, 0, 1, 't-harbour');
INSERT INTO runs VALUES ('r-scribe-1', 'scribe', 1794333480, 5, 'News Bearer', 5, 4020, 1, NULL, 'Condensed Dev Weekly for the briefing');
INSERT INTO cause_refs VALUES ('r-scribe-1', 0, 1, 't-dev-weekly');
INSERT INTO run_steps VALUES ('r-scribe-1', 0, 'Read', 'The newsletter', 1);
INSERT INTO step_refs VALUES ('r-scribe-1', 0, 0, 1, 't-dev-weekly');
INSERT INTO run_steps VALUES ('r-scribe-1', 1, 'Wrote', 'Two lines for the evening briefing', 2);
INSERT INTO step_refs VALUES ('r-scribe-1', 1, 0, 1, 't-dev-weekly');
INSERT INTO agents VALUES ('news-bearer', 3, 'News Bearer', 'Write a briefing at the set times: what agents did, what waits for you, newsletters and updates condensed, news on the topics you follow, and today’s calendar.
Show it in Briefing and notify me.', 1, '', 'Claude Sonnet', 1, 3, NULL, 0, NULL, 1794390600, 'Off until you add a model key', 2, 0);
INSERT INTO agent_times VALUES ('news-bearer', 450);
INSERT INTO agent_times VALUES ('news-bearer', 1080);
INSERT INTO agent_permissions VALUES ('news-bearer', 'write-briefing', 3, NULL);
INSERT INTO agent_permissions VALUES ('news-bearer', 'mail-briefing', 1, NULL);
INSERT INTO agent_accounts VALUES ('news-bearer', 'personal');
INSERT INTO agent_accounts VALUES ('news-bearer', 'laurentian');
INSERT INTO agent_accounts VALUES ('news-bearer', 'icloud');
INSERT INTO agent_stats VALUES ('news-bearer', 0, '1', 'briefing today');
INSERT INTO agent_stats VALUES ('news-bearer', 1, '2', 'newsletters condensed');
INSERT INTO agent_stats VALUES ('news-bearer', 2, '7:30', 'last run');
INSERT INTO runs VALUES ('r-news-bearer-5', 'news-bearer', 1794382200, 3, 'Daily at 7:30', 10, 9800, 1, NULL, 'Morning briefing ready');
INSERT INTO run_steps VALUES ('r-news-bearer-5', 0, 'Read', 'What agents did and what waits for you', 1);
INSERT INTO run_steps VALUES ('r-news-bearer-5', 1, 'Read', 'Newsletters and updates', 1);
INSERT INTO step_refs VALUES ('r-news-bearer-5', 1, 0, 1, 't-pragmatic-digest');
INSERT INTO step_refs VALUES ('r-news-bearer-5', 1, 1, 1, 't-parcel');
INSERT INTO run_steps VALUES ('r-news-bearer-5', 2, 'Read', 'News on 3 topics', 1);
INSERT INTO run_steps VALUES ('r-news-bearer-5', 3, 'Read', 'Today’s calendar', 1);
INSERT INTO step_refs VALUES ('r-news-bearer-5', 3, 0, 2, 'e-lecture-wed');
INSERT INTO step_refs VALUES ('r-news-bearer-5', 3, 1, 2, 'e-slides');
INSERT INTO run_steps VALUES ('r-news-bearer-5', 4, 'Wrote', 'Briefing for Wednesday morning', 2);
INSERT INTO run_steps VALUES ('r-news-bearer-5', 5, 'Notified', 'You', 2);
INSERT INTO runs VALUES ('r-news-bearer-4', 'news-bearer', 1794333600, 3, 'Daily at 18:00', 10, 9800, 1, NULL, 'Evening briefing ready');
INSERT INTO run_steps VALUES ('r-news-bearer-4', 0, 'Read', 'What agents did and what waits for you', 1);
INSERT INTO run_steps VALUES ('r-news-bearer-4', 1, 'Read', 'Newsletters and updates', 1);
INSERT INTO step_refs VALUES ('r-news-bearer-4', 1, 0, 1, 't-pragmatic-digest');
INSERT INTO step_refs VALUES ('r-news-bearer-4', 1, 1, 1, 't-parcel');
INSERT INTO run_steps VALUES ('r-news-bearer-4', 2, 'Read', 'News on 3 topics', 1);
INSERT INTO run_steps VALUES ('r-news-bearer-4', 3, 'Wrote', 'Briefing for Tuesday evening', 2);
INSERT INTO run_steps VALUES ('r-news-bearer-4', 4, 'Notified', 'You', 2);
INSERT INTO runs VALUES ('r-news-bearer-3', 'news-bearer', 1794295800, 3, 'Daily at 7:30', 10, 9800, 1, NULL, 'Morning briefing ready');
INSERT INTO run_steps VALUES ('r-news-bearer-3', 0, 'Read', 'What agents did and what waits for you', 1);
INSERT INTO run_steps VALUES ('r-news-bearer-3', 1, 'Read', 'Newsletters and updates', 1);
INSERT INTO step_refs VALUES ('r-news-bearer-3', 1, 0, 1, 't-pragmatic-digest');
INSERT INTO step_refs VALUES ('r-news-bearer-3', 1, 1, 1, 't-parcel');
INSERT INTO run_steps VALUES ('r-news-bearer-3', 2, 'Read', 'News on 3 topics', 1);
INSERT INTO run_steps VALUES ('r-news-bearer-3', 3, 'Wrote', 'Briefing for Tuesday morning', 2);
INSERT INTO run_steps VALUES ('r-news-bearer-3', 4, 'Notified', 'You', 2);
INSERT INTO runs VALUES ('r-news-bearer-2', 'news-bearer', 1794247200, 3, 'Daily at 18:00', 10, 9800, 1, NULL, 'Evening briefing ready');
INSERT INTO run_steps VALUES ('r-news-bearer-2', 0, 'Read', 'What agents did and what waits for you', 1);
INSERT INTO run_steps VALUES ('r-news-bearer-2', 1, 'Read', 'Newsletters and updates', 1);
INSERT INTO step_refs VALUES ('r-news-bearer-2', 1, 0, 1, 't-pragmatic-digest');
INSERT INTO step_refs VALUES ('r-news-bearer-2', 1, 1, 1, 't-parcel');
INSERT INTO run_steps VALUES ('r-news-bearer-2', 2, 'Read', 'News on 3 topics', 1);
INSERT INTO run_steps VALUES ('r-news-bearer-2', 3, 'Wrote', 'Briefing for Monday evening', 2);
INSERT INTO run_steps VALUES ('r-news-bearer-2', 4, 'Notified', 'You', 2);
INSERT INTO runs VALUES ('r-news-bearer-1', 'news-bearer', 1794209400, 3, 'Daily at 7:30', 10, 9800, 1, NULL, 'Morning briefing ready');
INSERT INTO run_steps VALUES ('r-news-bearer-1', 0, 'Read', 'What agents did and what waits for you', 1);
INSERT INTO run_steps VALUES ('r-news-bearer-1', 1, 'Read', 'Newsletters and updates', 1);
INSERT INTO step_refs VALUES ('r-news-bearer-1', 1, 0, 1, 't-pragmatic-digest');
INSERT INTO step_refs VALUES ('r-news-bearer-1', 1, 1, 1, 't-parcel');
INSERT INTO run_steps VALUES ('r-news-bearer-1', 2, 'Read', 'News on 3 topics', 1);
INSERT INTO run_steps VALUES ('r-news-bearer-1', 3, 'Wrote', 'Briefing for Monday morning', 2);
INSERT INTO run_steps VALUES ('r-news-bearer-1', 4, 'Notified', 'You', 2);
INSERT INTO agents VALUES ('assistant', 4, 'Assistant', 'Answer my questions about my mail and calendar across every account. Read and search on your own.
Propose any change and follow my permissions for it.', 1, '', 'Claude Sonnet', 1, 5, NULL, NULL, NULL, 1794390600, 'Off until you add a model key', 3, 1);
INSERT INTO agent_permissions VALUES ('assistant', 'read-search', 3, NULL);
INSERT INTO agent_permissions VALUES ('assistant', 'draft-replies', 3, NULL);
INSERT INTO agent_permissions VALUES ('assistant', 'send', 2, NULL);
INSERT INTO agent_permissions VALUES ('assistant', 'add-events', 2, NULL);
INSERT INTO agent_permissions VALUES ('assistant', 'delete', 1, NULL);
INSERT INTO agent_accounts VALUES ('assistant', 'personal');
INSERT INTO agent_accounts VALUES ('assistant', 'laurentian');
INSERT INTO agent_accounts VALUES ('assistant', 'icloud');
INSERT INTO agent_stats VALUES ('assistant', 0, '4', 'questions this week');
INSERT INTO agent_stats VALUES ('assistant', 1, '1', 'waiting for you');
INSERT INTO agent_stats VALUES ('assistant', 2, '9:48', 'last run');
INSERT INTO runs VALUES ('r-assistant-4', 'assistant', 1794390480, 4, 'c-assistant-4', 6, 5120, 2, NULL, 'Drafted a reply to Mira · waiting for you');
INSERT INTO run_steps VALUES ('r-assistant-4', 0, 'Read', 'Mira’s thread', 1);
INSERT INTO step_refs VALUES ('r-assistant-4', 0, 0, 1, 't-slides');
INSERT INTO run_steps VALUES ('r-assistant-4', 1, 'Drafted', 'Reply to Mira', 2);
INSERT INTO step_refs VALUES ('r-assistant-4', 1, 0, 1, 't-slides');
INSERT INTO run_steps VALUES ('r-assistant-4', 2, 'Send', 'Reply to Mira', 3);
INSERT INTO step_refs VALUES ('r-assistant-4', 2, 0, 3, 'a-assistant-mira');
INSERT INTO runs VALUES ('r-assistant-3', 'assistant', 1794385200, 4, 'c-assistant-3', 3, 2840, 1, NULL, 'Answered when and where your exam is');
INSERT INTO run_steps VALUES ('r-assistant-3', 0, 'Searched', 'Calendars and mail for COSC 4456 exam', 1);
INSERT INTO step_refs VALUES ('r-assistant-3', 0, 0, 2, 'e-exam');
INSERT INTO step_refs VALUES ('r-assistant-3', 0, 1, 1, 't-okafor');
INSERT INTO run_steps VALUES ('r-assistant-3', 1, 'Answered', 'Thursday 14:00 in FA-224', 2);
INSERT INTO runs VALUES ('r-assistant-2', 'assistant', 1794384720, 4, 'c-assistant-2', 8, 7310, 1, NULL, 'Answered what you owe this week');
INSERT INTO run_steps VALUES ('r-assistant-2', 0, 'Read', '9 threads from this week', 1);
INSERT INTO step_refs VALUES ('r-assistant-2', 0, 0, 1, 't-slides');
INSERT INTO step_refs VALUES ('r-assistant-2', 0, 1, 1, 't-northline');
INSERT INTO run_steps VALUES ('r-assistant-2', 1, 'Answered', 'Two things you owe', 2);
INSERT INTO runs VALUES ('r-assistant-1', 'assistant', 1794258840, 4, 'c-assistant-1', 41, 48200, 5, 'Stopped at the limit on mails one turn may read', 'Stopped at the read limit');
INSERT INTO run_steps VALUES ('r-assistant-1', 0, 'Read', 'Receipts from this year', 1);
INSERT INTO step_refs VALUES ('r-assistant-1', 0, 0, 1, 't-ridgeway');
INSERT INTO turns VALUES ('c-assistant-1', 'assistant', 1794225600, 'How much did I spend on everything this year?', 5, 'Stopped at the limit on mails one turn may read. Ask about a shorter span, such as this month', NULL);
INSERT INTO turn_refs VALUES ('c-assistant-2', 0, 1, 't-slides');
INSERT INTO turn_refs VALUES ('c-assistant-2', 1, 1, 't-northline');
INSERT INTO turns VALUES ('c-assistant-2', 'assistant', 1794384720, 'What do I still owe people this week?', 3, 'Two things. Mira asked you to take section 3 of the deck, the results, by tonight so the group can rehearse tomorrow morning.

Dana at Northline Talent wants two time windows for the interview loop. Job application tracker drafted a reply, and it waits for your approval.', NULL);
INSERT INTO turn_refs VALUES ('c-assistant-3', 0, 2, 'e-exam');
INSERT INTO turn_refs VALUES ('c-assistant-3', 1, 1, 't-okafor');
INSERT INTO turns VALUES ('c-assistant-3', 'assistant', 1794385200, 'When and where is my COSC 4456 exam?', 3, 'Thursday, Nov 12, 14:00 to 16:00 in FA-224. Prof. Okafor moved it from FA-110.', NULL);
INSERT INTO turns VALUES ('c-assistant-4', 'assistant', 1794390480, 'Tell Mira I’ll have section 3 done by tonight', 2, NULL, 'a-assistant-mira');
INSERT INTO agents VALUES ('job-tracker', 5, 'Job application tracker', NULL, 1, 'Watch for mail about my job applications: confirmations, interview invites, rejections and offers.
Label each one with the company and stage, and keep my Applications sheet up to date.
For interview invites, draft a reply with two time windows from my calendar.', 'Claude Sonnet', 1, 4, NULL, NULL, 3600, 1794390300, 'Off until you add a model key', 1, 1);
INSERT INTO agent_permissions VALUES ('job-tracker', 'read-label', 3, NULL);
INSERT INTO agent_permissions VALUES ('job-tracker', 'update-applications-sheet', 3, NULL);
INSERT INTO agent_permissions VALUES ('job-tracker', 'draft-replies', 3, NULL);
INSERT INTO agent_permissions VALUES ('job-tracker', 'send', 2, NULL);
INSERT INTO agent_triggers VALUES ('job-tracker', 'sorted');
INSERT INTO agent_accounts VALUES ('job-tracker', 'personal');
INSERT INTO agent_accounts VALUES ('job-tracker', 'laurentian');
INSERT INTO agent_stats VALUES ('job-tracker', 0, '24', 'labelled this month');
INSERT INTO agent_stats VALUES ('job-tracker', 1, '1', 'waiting for you');
INSERT INTO agent_stats VALUES ('job-tracker', 2, '9:45', 'last run');
INSERT INTO result_items VALUES ('ri-job-tracker-0-0', 'job-tracker', 'Drafted', 0, 0, 1794390300, 'Reply to Northline Talent', 'Two interview windows from your calendar, waiting for your approval');
INSERT INTO item_refs VALUES ('ri-job-tracker-0-0', 0, 3, 'a-northline');
INSERT INTO item_refs VALUES ('ri-job-tracker-0-0', 1, 1, 't-northline');
INSERT INTO result_items VALUES ('ri-job-tracker-1-0', 'job-tracker', 'Logged', 1, 0, 1794390300, 'Northline Talent · Interview', 'Stage updated in Applications');
INSERT INTO item_refs VALUES ('ri-job-tracker-1-0', 0, 1, 't-northline');
INSERT INTO result_items VALUES ('ri-job-tracker-1-1', 'job-tracker', 'Logged', 1, 1, 1794386460, 'Harbour Labs · Applied', 'Row added to Applications');
INSERT INTO item_refs VALUES ('ri-job-tracker-1-1', 0, 1, 't-harbour');
INSERT INTO result_items VALUES ('ri-job-tracker-1-2', 'job-tracker', 'Logged', 1, 2, 1794052800, 'Ostra Robotics · Applied', 'Row added to Applications');
INSERT INTO item_refs VALUES ('ri-job-tracker-1-2', 0, 1, 't-ostra');
INSERT INTO runs VALUES ('r-job-tracker-5', 'job-tracker', 1794390300, 1, NULL, 9, 6380, 2, NULL, 'Drafted 1 · waiting for you');
INSERT INTO cause_refs VALUES ('r-job-tracker-5', 0, 1, 't-northline');
INSERT INTO run_steps VALUES ('r-job-tracker-5', 0, 'Read', 'Northline Talent’s reply', 1);
INSERT INTO step_refs VALUES ('r-job-tracker-5', 0, 0, 1, 't-northline');
INSERT INTO run_steps VALUES ('r-job-tracker-5', 1, 'Labelled', 'Northline · Interview', 2);
INSERT INTO step_refs VALUES ('r-job-tracker-5', 1, 0, 1, 't-northline');
INSERT INTO run_steps VALUES ('r-job-tracker-5', 2, 'Updated', 'Applications: Northline to Interview', 2);
INSERT INTO run_steps VALUES ('r-job-tracker-5', 3, 'Read', 'Your calendar for next week', 1);
INSERT INTO run_steps VALUES ('r-job-tracker-5', 4, 'Drafted', 'Reply to Northline Talent', 2);
INSERT INTO step_refs VALUES ('r-job-tracker-5', 4, 0, 1, 't-northline');
INSERT INTO run_steps VALUES ('r-job-tracker-5', 5, 'Send', 'Reply to Northline Talent', 3);
INSERT INTO step_refs VALUES ('r-job-tracker-5', 5, 0, 3, 'a-northline');
INSERT INTO runs VALUES ('r-job-tracker-4', 'job-tracker', 1794387600, 3, 'Every hour', 1, 410, 1, NULL, 'Nothing new');
INSERT INTO run_steps VALUES ('r-job-tracker-4', 0, 'Read', 'For you since 8:00', 1);
INSERT INTO runs VALUES ('r-job-tracker-3', 'job-tracker', 1794386460, 1, NULL, 5, 2950, 1, NULL, 'Logged Harbour Labs application');
INSERT INTO cause_refs VALUES ('r-job-tracker-3', 0, 1, 't-harbour');
INSERT INTO run_steps VALUES ('r-job-tracker-3', 0, 'Read', 'Harbour Labs’ confirmation', 1);
INSERT INTO step_refs VALUES ('r-job-tracker-3', 0, 0, 1, 't-harbour');
INSERT INTO run_steps VALUES ('r-job-tracker-3', 1, 'Labelled', 'Harbour Labs · Applied', 2);
INSERT INTO step_refs VALUES ('r-job-tracker-3', 1, 0, 1, 't-harbour');
INSERT INTO run_steps VALUES ('r-job-tracker-3', 2, 'Logged', 'Row added to Applications', 2);
INSERT INTO runs VALUES ('r-job-tracker-2', 'job-tracker', 1794384000, 3, 'Every hour', 1, 390, 1, NULL, 'Nothing new');
INSERT INTO run_steps VALUES ('r-job-tracker-2', 0, 'Read', 'For you since 7:00', 1);
INSERT INTO runs VALUES ('r-job-tracker-1', 'job-tracker', 1794049320, 1, NULL, 4, 2710, 1, NULL, 'Logged Ostra Robotics application');
INSERT INTO cause_refs VALUES ('r-job-tracker-1', 0, 1, 't-ostra');
INSERT INTO run_steps VALUES ('r-job-tracker-1', 0, 'Read', 'Your application to Ostra Robotics', 1);
INSERT INTO step_refs VALUES ('r-job-tracker-1', 0, 0, 1, 't-ostra');
INSERT INTO run_steps VALUES ('r-job-tracker-1', 1, 'Labelled', 'Ostra Robotics · Applied', 2);
INSERT INTO step_refs VALUES ('r-job-tracker-1', 1, 0, 1, 't-ostra');
INSERT INTO run_steps VALUES ('r-job-tracker-1', 2, 'Logged', 'Row added to Applications', 2);
INSERT INTO agents VALUES ('receipts-clerk', 6, 'Receipts clerk', NULL, 1, 'Find receipts and invoices in every account.
Move them to Receipts and add vendor, amount and date to my budget sheet.
Never reply or forward.', 'Claude Haiku', 1, 1, NULL, NULL, NULL, 1794390600, 'Off until you add a model key', 1, 1);
INSERT INTO agent_permissions VALUES ('receipts-clerk', 'move-folders', 3, NULL);
INSERT INTO agent_permissions VALUES ('receipts-clerk', 'update-budget-sheet', 3, NULL);
INSERT INTO agent_permissions VALUES ('receipts-clerk', 'reply-forward', 1, NULL);
INSERT INTO agent_triggers VALUES ('receipts-clerk', 'new-mail');
INSERT INTO agent_accounts VALUES ('receipts-clerk', 'personal');
INSERT INTO agent_accounts VALUES ('receipts-clerk', 'laurentian');
INSERT INTO agent_accounts VALUES ('receipts-clerk', 'icloud');
INSERT INTO agent_stats VALUES ('receipts-clerk', 0, '86', 'filed this month');
INSERT INTO agent_stats VALUES ('receipts-clerk', 1, '100%', 'runs with no questions');
INSERT INTO agent_stats VALUES ('receipts-clerk', 2, '8:30', 'last filed');
INSERT INTO result_items VALUES ('ri-receipts-clerk-0-0', 'receipts-clerk', 'Filed', 0, 0, 1794385800, 'Invoice #4471 · Ridgeway Hosting', 'Moved to Receipts; vendor, amount and date added to your budget sheet');
INSERT INTO item_refs VALUES ('ri-receipts-clerk-0-0', 0, 1, 't-ridgeway');
INSERT INTO runs VALUES ('r-receipts-clerk-3', 'receipts-clerk', 1794390060, 1, NULL, 1, 520, 1, NULL, 'Nothing to file');
INSERT INTO cause_refs VALUES ('r-receipts-clerk-3', 0, 1, 't-okafor');
INSERT INTO run_steps VALUES ('r-receipts-clerk-3', 0, 'Read', 'Prof. Okafor’s mail: not a receipt', 1);
INSERT INTO step_refs VALUES ('r-receipts-clerk-3', 0, 0, 1, 't-okafor');
INSERT INTO runs VALUES ('r-receipts-clerk-2', 'receipts-clerk', 1794385800, 1, NULL, 6, 1870, 1, NULL, 'Filed Ridgeway Hosting invoice');
INSERT INTO cause_refs VALUES ('r-receipts-clerk-2', 0, 1, 't-ridgeway');
INSERT INTO run_steps VALUES ('r-receipts-clerk-2', 0, 'Read', 'Invoice #4471', 1);
INSERT INTO step_refs VALUES ('r-receipts-clerk-2', 0, 0, 1, 't-ridgeway');
INSERT INTO run_steps VALUES ('r-receipts-clerk-2', 1, 'Filed', 'Invoice #4471 to Receipts', 2);
INSERT INTO step_refs VALUES ('r-receipts-clerk-2', 1, 0, 1, 't-ridgeway');
INSERT INTO run_steps VALUES ('r-receipts-clerk-2', 2, 'Logged', 'Row added to the budget sheet', 2);
INSERT INTO runs VALUES ('r-receipts-clerk-1', 'receipts-clerk', 1794338520, 1, NULL, 1, 480, 1, NULL, 'Nothing to file');
INSERT INTO cause_refs VALUES ('r-receipts-clerk-1', 0, 1, 't-parcel');
INSERT INTO run_steps VALUES ('r-receipts-clerk-1', 0, 'Read', 'Parcelway’s update: not a receipt', 1);
INSERT INTO step_refs VALUES ('r-receipts-clerk-1', 0, 0, 1, 't-parcel');
INSERT INTO agents VALUES ('follow-up-nudger', 7, 'Follow-up nudger', NULL, 1, 'If I sent something that needs an answer and nobody replied in 3 days, draft a short, friendly nudge.
Ask before sending to anyone I haven’t emailed before.', 'Claude Sonnet', 1, 3, NULL, 1, NULL, 1794390600, 'Off until you add a model key', 1, 1);
INSERT INTO agent_times VALUES ('follow-up-nudger', 540);
INSERT INTO agent_permissions VALUES ('follow-up-nudger', 'read-sent', 3, NULL);
INSERT INTO agent_permissions VALUES ('follow-up-nudger', 'draft-replies', 3, NULL);
INSERT INTO agent_permissions VALUES ('follow-up-nudger', 'send', 2, NULL);
INSERT INTO agent_accounts VALUES ('follow-up-nudger', 'personal');
INSERT INTO agent_accounts VALUES ('follow-up-nudger', 'laurentian');
INSERT INTO agent_stats VALUES ('follow-up-nudger', 0, '12', 'nudges sent');
INSERT INTO agent_stats VALUES ('follow-up-nudger', 1, '1', 'waiting for you');
INSERT INTO agent_stats VALUES ('follow-up-nudger', 2, '9:00', 'last run');
INSERT INTO result_items VALUES ('ri-follow-up-nudger-0-0', 'follow-up-nudger', 'Nudges', 0, 0, 1794387600, 'Nudge to Ostra Robotics recruiting', 'No reply in 4 days; waiting for your approval');
INSERT INTO item_refs VALUES ('ri-follow-up-nudger-0-0', 0, 3, 'a-ostra');
INSERT INTO item_refs VALUES ('ri-follow-up-nudger-0-0', 1, 1, 't-ostra');
INSERT INTO runs VALUES ('r-follow-up-nudger-3', 'follow-up-nudger', 1794387600, 3, 'Weekdays at 9:00', 7, 4630, 2, NULL, 'Drafted 1 nudge · waiting for you');
INSERT INTO run_steps VALUES ('r-follow-up-nudger-3', 0, 'Read', 'Your sent mail from the last two weeks', 1);
INSERT INTO run_steps VALUES ('r-follow-up-nudger-3', 1, 'Read', 'No reply from Ostra Robotics in 4 days', 1);
INSERT INTO step_refs VALUES ('r-follow-up-nudger-3', 1, 0, 1, 't-ostra');
INSERT INTO run_steps VALUES ('r-follow-up-nudger-3', 2, 'Drafted', 'Nudge to Ostra Robotics recruiting', 2);
INSERT INTO step_refs VALUES ('r-follow-up-nudger-3', 2, 0, 1, 't-ostra');
INSERT INTO run_steps VALUES ('r-follow-up-nudger-3', 3, 'Send', 'Nudge to Ostra Robotics recruiting', 3);
INSERT INTO step_refs VALUES ('r-follow-up-nudger-3', 3, 0, 3, 'a-ostra');
INSERT INTO runs VALUES ('r-follow-up-nudger-2', 'follow-up-nudger', 1794301200, 3, 'Weekdays at 9:00', 3, 2140, 1, NULL, 'Nothing to nudge');
INSERT INTO run_steps VALUES ('r-follow-up-nudger-2', 0, 'Read', 'Your sent mail from the last two weeks', 1);
INSERT INTO runs VALUES ('r-follow-up-nudger-1', 'follow-up-nudger', 1794214800, 3, 'Weekdays at 9:00', 3, 2090, 1, NULL, 'Nothing to nudge');
INSERT INTO run_steps VALUES ('r-follow-up-nudger-1', 0, 'Read', 'Your sent mail from the last two weeks', 1);
INSERT INTO agents VALUES ('day-summary', 8, 'Day summary', NULL, 1, 'Give me a brief summary of what came in today and what still needs an answer.', 'Claude Sonnet', 1, 3, NULL, 1, NULL, 1794390600, 'Off until you add a model key', 1, 1);
INSERT INTO agent_times VALUES ('day-summary', 1020);
INSERT INTO agent_permissions VALUES ('day-summary', 'read-mail', 3, NULL);
INSERT INTO agent_accounts VALUES ('day-summary', 'personal');
INSERT INTO agent_accounts VALUES ('day-summary', 'laurentian');
INSERT INTO agent_accounts VALUES ('day-summary', 'icloud');
INSERT INTO agent_stats VALUES ('day-summary', 0, '3', 'summaries this week');
INSERT INTO agent_stats VALUES ('day-summary', 1, '9:30', 'last answer');
INSERT INTO result_items VALUES ('ri-day-summary-0-0', 'day-summary', 'Summaries', 0, 0, 1794389400, 'This morning so far', 'Three things need you: Mira’s section 3, Northline’s interview times and Thursday’s exam room.');
INSERT INTO item_refs VALUES ('ri-day-summary-0-0', 0, 1, 't-slides');
INSERT INTO item_refs VALUES ('ri-day-summary-0-0', 1, 1, 't-northline');
INSERT INTO item_refs VALUES ('ri-day-summary-0-0', 2, 1, 't-okafor');
INSERT INTO result_items VALUES ('ri-day-summary-0-1', 'day-summary', 'Summaries', 0, 1, 1794312000, 'Tuesday', '41 mails came in; the two that needed an answer got one.');
INSERT INTO runs VALUES ('r-day-summary-5', 'day-summary', 1794390600, 4, 'c-day-summary-3', NULL, 2300, 3, NULL, 'Reading your mail');
INSERT INTO run_steps VALUES ('r-day-summary-5', 0, 'Read', 'Mail since 9:30', 1);
INSERT INTO step_refs VALUES ('r-day-summary-5', 0, 0, 1, 't-okafor');
INSERT INTO runs VALUES ('r-day-summary-4', 'day-summary', 1794389400, 4, 'c-day-summary-2', 7, 6120, 1, NULL, 'Summarized this morning');
INSERT INTO run_steps VALUES ('r-day-summary-4', 0, 'Read', '14 mails since midnight', 1);
INSERT INTO step_refs VALUES ('r-day-summary-4', 0, 0, 1, 't-slides');
INSERT INTO step_refs VALUES ('r-day-summary-4', 0, 1, 1, 't-northline');
INSERT INTO step_refs VALUES ('r-day-summary-4', 0, 2, 1, 't-okafor');
INSERT INTO run_steps VALUES ('r-day-summary-4', 1, 'Answered', 'Three things need you', 2);
INSERT INTO runs VALUES ('r-day-summary-3', 'day-summary', 1794380520, 4, 'c-day-summary-1', 30, 0, 4, 'Anthropic did not answer in time. Nothing was changed', 'Failed');
INSERT INTO run_steps VALUES ('r-day-summary-3', 0, 'Read', 'Mail since 18:00 yesterday', 1);
INSERT INTO runs VALUES ('r-day-summary-2', 'day-summary', 1794330000, 3, 'Weekdays at 17:00', 9, 8040, 1, NULL, 'Summary ready');
INSERT INTO run_steps VALUES ('r-day-summary-2', 0, 'Read', '41 mails from Tuesday', 1);
INSERT INTO run_steps VALUES ('r-day-summary-2', 1, 'Wrote', 'Tuesday’s summary', 2);
INSERT INTO runs VALUES ('r-day-summary-1', 'day-summary', 1794243600, 3, 'Weekdays at 17:00', 8, 7310, 1, NULL, 'Summary ready');
INSERT INTO run_steps VALUES ('r-day-summary-1', 0, 'Read', '37 mails from Monday', 1);
INSERT INTO run_steps VALUES ('r-day-summary-1', 1, 'Wrote', 'Monday’s summary', 2);
INSERT INTO turns VALUES ('c-day-summary-1', 'day-summary', 1794380520, 'Summarize last night’s mail', 4, 'Anthropic did not answer in time. Nothing was changed', NULL);
INSERT INTO turn_refs VALUES ('c-day-summary-2', 0, 1, 't-slides');
INSERT INTO turn_refs VALUES ('c-day-summary-2', 1, 1, 't-northline');
INSERT INTO turn_refs VALUES ('c-day-summary-2', 2, 1, 't-okafor');
INSERT INTO turns VALUES ('c-day-summary-2', 'day-summary', 1794389400, 'Summarize this morning so far', 3, 'Three things need you: Mira wants section 3 of the deck by tonight, Dana at Northline Talent is waiting on interview times, and Prof. Okafor moved Thursday’s exam to FA-224.

The rest were updates and newsletters, already sorted.', NULL);
INSERT INTO turns VALUES ('c-day-summary-3', 'day-summary', 1794390600, 'Anything new since then?', 1, 'Reading your mail', NULL);
INSERT INTO approvals VALUES ('a-assistant-mira', 'assistant', 1794390480, 'Reply to Mira', '“Hi Mira, I’ll have section 3 done by tonight…”', 'Assistant asks before sending', 1, 't-slides', 1, 'Hi Mira,

I’ll have section 3, the results, done by tonight, so we’re set for the rehearsal tomorrow morning.

Ali', NULL);
INSERT INTO approvals VALUES ('a-northline', 'job-tracker', 1794390300, 'Reply to Northline Talent', 'Offers Tuesday 13:00–16:00 or Thursday 10:00–13:00 for the interview loop.', 'Gmail accounts always ask before sending for now', 1, 't-northline', 1, 'Hi Dana,

That sounds great. Tuesday 13:00–16:00 or Thursday 10:00–13:00 (Eastern) both work for me.

Thanks,
Ali', NULL);
INSERT INTO approvals VALUES ('a-ostra', 'follow-up-nudger', 1794387600, 'Nudge Ostra Robotics recruiting', '“Hi Sam, just checking in on my application from last week…”', 'No reply in 4 days', 1, 't-ostra', 1, 'Hi Sam,

Just checking in on my application for the Platform Engineer role from last week. Happy to share anything else that helps.

Thanks,
Ali', NULL);
INSERT INTO approvals VALUES ('a-jobstream', 'gatekeeper', 1794225600, 'Unsubscribe from Jobstream weekly?', 'Sent 46 mails to Laurentian in the last 90 days. You opened 2.', 'Found when Laurentian was connected', 2, NULL, 1, '', NULL);
INSERT INTO approvals VALUES ('a-campus-store', 'gatekeeper', 1794225600, 'Unsubscribe from Campus Store deals?', 'Sent 31 mails to Laurentian in the last 90 days. You opened none.', 'Found when Laurentian was connected', 2, NULL, 1, '', NULL);
INSERT INTO editions VALUES ('b-wed-morning', 1794382200, 'Wednesday 7:30 · written by News Bearer', 1, 'Today', 1);
INSERT INTO edition_lines VALUES ('b-wed-morning', 1, 0, 'Gatekeeper', 'sorted 12 mails that came in overnight: 2 for you, 4 updates, 6 to read.');
INSERT INTO edition_lines VALUES ('b-wed-morning', 1, 1, 'Security Guard', 'caught 1 phishing mail pretending to be your mail provider, blocked the sender and reported it to Gmail.');
INSERT INTO edition_lines VALUES ('b-wed-morning', 2, 0, 'The Pragmatic Digest', 'Three long reads on queues, caches and backpressure. The short version: bound every queue, and let slow consumers push back instead of dropping work.');
INSERT INTO edition_lines VALUES ('b-wed-morning', 3, 0, 'Parcelway', 'your package was delivered yesterday at 18:40.');
INSERT INTO edition_waiting VALUES ('b-wed-morning', 0, 'a-jobstream', 'Unsubscribe from Jobstream weekly?', 'Gatekeeper · 46 mails in 90 days, you opened 2');
INSERT INTO edition_waiting VALUES ('b-wed-morning', 1, 'a-campus-store', 'Unsubscribe from Campus Store deals?', 'Gatekeeper · 31 mails in 90 days, you opened none');
INSERT INTO edition_events VALUES ('b-wed-morning', 0, '10:00', 'COSC 4456 lecture', 'FA-110 · Laurentian');
INSERT INTO edition_events VALUES ('b-wed-morning', 1, '19:00', 'Section 3 slides', 'Focus time · Personal');
INSERT INTO edition_news VALUES ('b-wed-morning', 'Distributed systems', 0, 'Why bounded queues beat retries under load', 'The Backlog');
INSERT INTO edition_news VALUES ('b-wed-morning', 'Distributed systems', 1, 'A field guide to consensus timeouts', 'Systems Margins');
INSERT INTO edition_news VALUES ('b-wed-morning', 'Climbing', 2, 'Winter bouldering league opens sign-ups', 'North Wall Notes');
INSERT INTO edition_news VALUES ('b-wed-morning', 'Climbing', 3, 'Five finger-strength drills without a hangboard', 'Crux Weekly');
INSERT INTO edition_news VALUES ('b-wed-morning', 'ML infrastructure jobs', 4, 'Inference platform teams are hiring again', 'Stackline Careers');
INSERT INTO edition_news VALUES ('b-wed-morning', 'ML infrastructure jobs', 5, 'What interview loops for infrastructure roles ask now', 'The Backlog');
INSERT INTO editions VALUES ('b-tue-evening', 1794333600, 'Tuesday 18:00 · written by News Bearer', 1, 'Tomorrow', 0);
INSERT INTO edition_lines VALUES ('b-tue-evening', 1, 0, 'Gatekeeper', 'sorted 41 new mails: 6 for you, 14 updates, 21 to read.');
INSERT INTO edition_lines VALUES ('b-tue-evening', 1, 1, 'Security Guard', 'caught 1 scam asking for a customs fee, blocked the sender and reported it to Outlook.');
INSERT INTO edition_lines VALUES ('b-tue-evening', 2, 0, 'Dev Weekly', 'Zig 0.15 notes are out. The long read revisits async I/O and argues for explicit event loops over hidden ones.');
INSERT INTO edition_lines VALUES ('b-tue-evening', 3, 0, 'Parcelway', 'your package is out for delivery, expected before 20:00.');
INSERT INTO edition_waiting VALUES ('b-tue-evening', 0, 'a-jobstream', 'Unsubscribe from Jobstream weekly?', 'Gatekeeper · 46 mails in 90 days, you opened 2');
INSERT INTO edition_waiting VALUES ('b-tue-evening', 1, 'a-campus-store', 'Unsubscribe from Campus Store deals?', 'Gatekeeper · 31 mails in 90 days, you opened none');
INSERT INTO edition_events VALUES ('b-tue-evening', 0, '10:00', 'COSC 4456 lecture', 'FA-110 · Laurentian');
INSERT INTO edition_events VALUES ('b-tue-evening', 1, '19:00', 'Section 3 slides', 'Focus time · Personal');
INSERT INTO edition_news VALUES ('b-tue-evening', 'Distributed systems', 0, 'Clock skew, explained with one diagram', 'Systems Margins');
INSERT INTO edition_news VALUES ('b-tue-evening', 'Climbing', 1, 'Route setters on reading a problem before touching it', 'Crux Weekly');
INSERT INTO edition_news VALUES ('b-tue-evening', 'ML infrastructure jobs', 2, 'New-grad infrastructure roles: what changed this fall', 'Stackline Careers');
INSERT INTO news_topics VALUES (0, 'Distributed systems');
INSERT INTO news_topics VALUES (1, 'Climbing');
INSERT INTO news_topics VALUES (2, 'ML infrastructure jobs');
INSERT INTO quarantine VALUES ('q-mailbox', 'security@acc0unt-verify.net', 'personal', 1794381120, 'Your mailbox will be closed today', 'Phishing: asks you to sign in on a look-alike page', 'Caught by Security Guard', 'The sender pretends to be your mail provider and links to a sign-in page on another domain. SPF and DKIM both failed.', 'Sender blocked and reported to Gmail', 'Dear user,

Your mailbox has exceeded its limit and will be closed today. Verify your account now to keep your messages.

[link removed]

Mail Support Team', 1796973120);
INSERT INTO quarantine VALUES ('q-customs', 'noreply@shipping-hold.info', 'laurentian', 1794312000, 'Customs fee unpaid for your parcel', 'Scam: asks for a payment to release a parcel', 'Caught by Security Guard', 'Asks for a small card payment to release a parcel. The domain was registered four days ago and you have no order with this sender.', 'Sender blocked and reported to Outlook', 'Your parcel is held at customs. Pay the outstanding fee of $2.99 within 24 hours or it will be returned.

[link removed]', 1796904000);
INSERT INTO quarantine VALUES ('q-deals', 'deals@megasavings.shop', 'icloud', 1793707200, 'Final hours: 80% off everything', 'Blocked sender', 'Blocked sender', 'You blocked this sender on Oct 2, so it came straight here without any agent reading it.', 'Already on your block list', 'Final hours! Everything 80% off, today only.

[link removed]', 1796299200);
INSERT INTO model_keys VALUES (1, 'Anthropic', 1);
INSERT INTO model_keys VALUES (2, 'OpenAI', 0);
INSERT INTO model_keys VALUES (3, 'Google Gemini', 1);
INSERT INTO model_keys VALUES (4, 'OpenRouter', 0);
INSERT INTO models VALUES ('Claude Sonnet', 0, 1);
INSERT INTO models VALUES ('Claude Haiku', 1, 1);
INSERT INTO models VALUES ('Gemini Pro', 2, 3);
INSERT INTO models VALUES ('GPT-5', 3, 2);
INSERT INTO models VALUES ('OpenRouter models', 4, 4);
INSERT INTO task_models VALUES ('replies', 0, 'Writing replies', 'AI compose and drafts', 'Claude Sonnet');
INSERT INTO task_models VALUES ('summaries', 1, 'Summaries', 'Scribe', 'Gemini Pro');
INSERT INTO task_models VALUES ('agents', 2, 'Agents', 'System agents and new agents', 'Claude Sonnet');
INSERT INTO default_permissions VALUES ('read-label', 3, NULL);
INSERT INTO default_permissions VALUES ('draft-replies', 3, NULL);
INSERT INTO default_permissions VALUES ('archive', 3, NULL);
INSERT INTO default_permissions VALUES ('send', 2, NULL);
INSERT INTO default_permissions VALUES ('unsubscribe', 2, NULL);
INSERT INTO default_permissions VALUES ('delete', 1, NULL);
INSERT INTO settings VALUES (1, 1, 0, 500000, 'Brief', 1, 30, 30);
-- Send cannot be Auto where an agent sees the Gmail account.
UPDATE agent_permissions SET ask_at_most = 'Can’t be Auto on Gmail yet, Google is still reviewing EveryMail' WHERE action_id = 'send' AND agent_id IN (SELECT agent_id FROM agent_accounts WHERE account_id = 'personal');
-- The agent half written in the builder.
INSERT INTO builder VALUES (1, 'Deadline keeper', 'When a professor emails about a deadline, add it to my calendar and remind me the day before.', 'daily-8', 900, 1);
INSERT INTO builder_triggers VALUES ('sorted', 1);
INSERT INTO builder_triggers VALUES ('event-approaching', 1);
INSERT INTO builder_filters VALUES ('sorted', 'into', 'for-you');
INSERT INTO builder_accounts VALUES ('personal');
INSERT INTO builder_accounts VALUES ('laurentian');
INSERT INTO builder_permissions VALUES ('read-label', 3, NULL);
INSERT INTO builder_permissions VALUES ('update-sheets', 1, NULL);
INSERT INTO builder_permissions VALUES ('calendar', 3, NULL);
INSERT INTO builder_permissions VALUES ('draft-replies', 2, NULL);
INSERT INTO builder_permissions VALUES ('send', 2, NULL);
INSERT INTO builder_permissions VALUES ('archive', 1, NULL);
INSERT INTO builder_permissions VALUES ('delete', 1, NULL);
INSERT INTO rules VALUES ('rule-laurentian', 0, 2, 'laurentian.ca', 1);
INSERT INTO rule_actions VALUES ('rule-laurentian', 0, 4, NULL, NULL);
INSERT INTO rules VALUES ('rule-receipts', 1, 3, NULL, 1);
INSERT INTO rule_words VALUES ('rule-receipts', 0, 'receipt');
INSERT INTO rule_words VALUES ('rule-receipts', 1, 'invoice');
INSERT INTO rule_actions VALUES ('rule-receipts', 0, 5, NULL, 'receipts-clerk');
INSERT INTO rules VALUES ('rule-newsletters', 2, 4, NULL, 1);
INSERT INTO rule_actions VALUES ('rule-newsletters', 0, 6, NULL, NULL);
INSERT INTO rules VALUES ('rule-new-link', 3, 5, NULL, 0);
INSERT INTO rule_actions VALUES ('rule-new-link', 0, 5, NULL, 'security-guard');
INSERT INTO blocked VALUES ('security@acc0unt-verify.net', 0, 'Security Guard', 1794381120);
INSERT INTO blocked VALUES ('noreply@shipping-hold.info', 1, 'Security Guard', 1794329280);
INSERT INTO blocked VALUES ('prizes@luckywinner-claims.biz', 2, 'Security Guard', 1793785800);
INSERT INTO blocked VALUES ('deals@megasavings.shop', 3, 'you', 1790934600);
UPDATE builder_permissions SET ask_at_most = 'Can’t be Auto on Gmail yet, Google is still reviewing EveryMail' WHERE action_id = 'send';
