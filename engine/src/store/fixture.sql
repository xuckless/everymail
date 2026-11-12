-- A small replica for the engine's tests, as of Wednesday, November 11 2026, 9:50 (1794390600).
-- Two accounts, Mail on Gmail and Work on Outlook; Sorter sees Mail only, Helper chats.

PRAGMA defer_foreign_keys = ON;

INSERT INTO accounts VALUES ('mail', 0, 'Mail', 'kai@gmail.com', 1, 'Gmail', 1, NULL, NULL, 1794390000, NULL);
INSERT INTO accounts VALUES ('work', 1, 'Work', 'kai@fernhill.org', 2, 'Outlook', 1, NULL, NULL, 1794390000, 'Kai');
INSERT INTO folders VALUES ('starred', 0, 'Starred', 1);
INSERT INTO folders VALUES ('sent', 1, 'Sent', 2);
INSERT INTO folders VALUES ('archive', 2, 'Archive', 3);
INSERT INTO folders VALUES ('bills', 3, 'Bills', 5);
INSERT INTO folders VALUES ('trash', 4, 'Trash', 6);
INSERT INTO folders VALUES ('quarantine', 5, 'Quarantine', 4);
INSERT INTO threads VALUES ('t-lake', 'work', 1, 1, 'Noor', 'Lake survey results', 1794390000, NULL, NULL, NULL, 1, NULL);
INSERT INTO messages VALUES ('m-lake', 't-lake', 0, 0, 'Noor', 'The survey is in', 1794390000, 'The survey is in.

Noor', NULL, 0);
INSERT INTO threads VALUES ('t-boat', 'mail', 1, 1, 'Ravi', 'Boat on Sunday', 1794380000, NULL, NULL, NULL, 0, NULL);
INSERT INTO thread_people VALUES ('t-boat', 0, 'Mo');
INSERT INTO messages VALUES ('m-boat-1', 't-boat', 0, 1, 'Kai', 'Who is in for Sunday', 1794370000, 'Who is in for Sunday?', NULL, 0);
INSERT INTO messages VALUES ('m-boat-2', 't-boat', 1, 0, 'Ravi', 'Count me in', 1794380000, 'Count me in.', NULL, 1);
INSERT INTO threads VALUES ('t-power', 'mail', 1, 2, 'Gridline Power', 'Your power bill', 1794300000, NULL, NULL, NULL, 1, NULL);
INSERT INTO messages VALUES ('m-power', 't-power', 0, 0, 'Gridline Power', 'Your bill is ready', 1794300000, 'Your bill is ready.', NULL, 0);
INSERT INTO models VALUES ('Model One', 0, 1);
INSERT INTO task_models VALUES ('replies', 0, 'Writing replies', 'AI compose', 'Model One');
INSERT INTO task_models VALUES ('agents', 1, 'Agents', 'New agents', 'Model One');
INSERT INTO model_keys VALUES (1, 'Anthropic', 1);
INSERT INTO actions VALUES ('read-label', 0, 'Read and label', NULL, 1);
INSERT INTO actions VALUES ('send', 1, 'Send', 'Mail leaves your account', 1);
INSERT INTO default_permissions VALUES ('read-label', 3, NULL);
INSERT INTO default_permissions VALUES ('send', 2, NULL);
INSERT INTO agents VALUES ('sorter', 0, 'Sorter', 'Sorts mail', 1, '', 'Model One', 1, 1, NULL, NULL, NULL, 1794390000, 'Off without a key', 1, 0);
INSERT INTO agent_permissions VALUES ('sorter', 'read-label', 3, NULL);
INSERT INTO agent_permissions VALUES ('sorter', 'send', 2, 'Not on Gmail');
INSERT INTO agent_accounts VALUES ('sorter', 'mail');
INSERT INTO runs VALUES ('r-sorter', 'sorter', 1794390000, 1, NULL, 2, 0, 1, NULL, 'Sorted 1');
INSERT INTO cause_refs VALUES ('r-sorter', 0, 1, 't-lake');
INSERT INTO run_steps VALUES ('r-sorter', 0, 'Sorted', 'into For you', 2);
INSERT INTO step_refs VALUES ('r-sorter', 0, 0, 1, 't-lake');
INSERT INTO agents VALUES ('helper', 1, 'Helper', NULL, 1, 'Answers questions', 'Model One', 1, 5, NULL, NULL, NULL, 1794390000, 'Off without a key', 3, 1);
INSERT INTO agent_accounts VALUES ('helper', 'mail');
INSERT INTO agent_accounts VALUES ('helper', 'work');
INSERT INTO turns VALUES ('turn-1', 'helper', 1794380000, 'What is new?', 3, 'One survey.', NULL);
INSERT INTO turn_refs VALUES ('turn-1', 0, 1, 't-lake');
INSERT INTO turns VALUES ('turn-2', 'helper', 1794385000, 'Reply to Noor', 2, NULL, 'a-noor');
INSERT INTO approvals VALUES ('a-noor', 'helper', 1794385000, 'Reply to Noor', 'Thanks for the survey', 'Sending asks first', 1, 't-lake', 1, 'Hi Noor,

Thanks.', NULL);
INSERT INTO quarantine VALUES ('q-prize', 'win@prize.top', 'mail', 1794300000, 'You won', 'Scam', 'Security Guard', 'A fake prize', 'Sender blocked', 'Claim now', 1796892000);
INSERT INTO blocked VALUES ('win@prize.top', 0, 'Security Guard', 1794300000);
INSERT INTO rules VALUES ('rule-work', 0, 2, 'fernhill.org', 1);
INSERT INTO rule_actions VALUES ('rule-work', 0, 4, NULL, NULL);
INSERT INTO calendars VALUES ('cal', 0, 'Calendar', 1, 'Google Calendar', 1, 1);
INSERT INTO settings VALUES (1, 1, 0, NULL, 'Brief', 1, 30, 30);
INSERT INTO builder VALUES (1, 'Bill filer', 'File bills', 'none', 900, 1);
INSERT INTO builder_accounts VALUES ('mail');
INSERT INTO builder_permissions VALUES ('read-label', 3, NULL);
INSERT INTO builder_permissions VALUES ('send', 2, 'Not on Gmail');
