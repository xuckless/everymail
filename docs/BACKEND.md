# Backend

How the Backend works: what is decided, and why. The rules every part obeys are in [AGENTS.md](../AGENTS.md). The tasks that build this are in [TODO-backend.md](../TODO-backend.md), and what is still undecided is there as "Decide:" tasks.

## Deployment

- The Backend runs on GCE next to the EveryAI services
- Keep the deployment lean: no extra infrastructure until a measured need

## Backend engine

`Bun.serve` for now. Fastify was dropped: with protobuf only on the wire its validation, serialization and plugins go unused

- Handlers are pure functions from a protobuf request to a protobuf response, registered in one RPC table that knows nothing about HTTP
- One thin adapter binds the table to the HTTP engine. It is the only code that knows about HTTP
- Later the HTTP engine is replaced by the Anole engine (the owner's Zig server, https://github.com/xuckless/anole) by replacing that adapter: no rewrite of application code is allowed or needed

Expected gain: higher throughput at a higher idle memory cost. Owner's benchmark, Bun vs Bun with Anole:

| Route      | req/s                   | p99 ms    | CPU µs/req | Memory MB |
| ---------- | ----------------------- | --------- | ---------- | --------- |
| plaintext  | 151.2k → 326.0k (2.16x) | 1.2 → 0.7 | 4.8 → 5.6  | 99 → 201  |
| json       | 151.1k → 333.5k (2.21x) | 1.2 → 0.7 | 4.7 → 5.6  | 103 → 195 |
| users/42   | 148.4k → 320.0k (2.16x) | 1.2 → 0.7 | 4.9 → 6.0  | 112 → 190 |
| POST items | 116.9k → 241.0k (2.06x) | 1.6 → 1.2 | 6.7 → 8.1  | 108 → 216 |

## Backend database

- PostgreSQL with pgvector, on the GCE VM, accessed from TypeScript
- Gate: Postgres uses less than 100 MB of memory when idle on the VM, measured as anonymous plus shared memory of all its processes, not page cache. If it does not fit, fall back to Turso accessed from TypeScript
- Run it tuned for a small box: small `shared_buffers`, few `max_connections` behind a small pool, JIT and parallel workers off

## Work model

Everything the Backend does on its own is a job: fetching mail, sending mail, snooze wake-ups, token refresh, the briefing, and every step of an agent run. Jobs go in at one end; executors that hold nothing between jobs take them out at the other

## Jobs

- One jobs table in Postgres is the queue. All state lives in Postgres: an executor can die at any point and lose nothing
- A job's status is an explicit enum: scheduled, ready, running (under a lease), waiting for approval, waiting for an event, paused, succeeded, failed, cancelled. A lease that expires returns the job to ready
- Waiting for an event covers a tool result, a provider batch result and another agent finishing
- Executors claim jobs in batches and never poll when idle: enqueueing signals the executor in process, and a timer sleeps until the next due time. `Bun.sql` cannot receive `LISTEN/NOTIFY`, so that is added only when the executor becomes a separate process
- No transaction and no executor is held open across an external call: claim and commit, call, then record and commit
- One mechanism, several lanes. Each job kind has its own concurrency cap, priority, retry policy and timeout, so slow agent steps never delay a send
- Columns hold what the claim query needs; the rest of the payload is protobuf
- A finished job is deleted; the run log is what is kept. The run log covers agent runs. A job of another kind that keeps failing shows on the thing it concerns: the account's status or the message's send state
- The run log is kept 90 days. The transcript, which holds mail content and model output, is deleted after 30 days; a parked run keeps its transcript until it resolves
- The executor runs inside the Backend process, behind an interface that allows a separate process later. A restart retries in-flight jobs from their start
- Every row carries a user id. Claiming is fair across users and capped per user, so one backlog cannot starve the rest. Sized for tens to hundreds of users on the one VM

## Time

- Postgres `now()` is the only source of "now". Client clocks are never trusted: clients send intents and the server stamps them
- Instants are stored in UTC. A scheduled time arrives as an absolute UTC instant, or as wall time plus an IANA zone from which the server computes the instant. Recurrence rules keep their zone, because "daily at 08:00" is not one instant
- Durations are in seconds everywhere in the Backend
- A proactive wakeup has a minimum interval of 900 seconds (15 minutes), with jitter
- After downtime a missed schedule runs once, not once per missed slot. If the previous run is still going when the next is due, the next is skipped
- A scheduled send, undo send (a 10 second delay on "send now") and snooze are the same thing: a job with a due time that can be edited or cancelled until it is claimed

## Side effects

- Delivery is at least once, so every side effect has an idempotency key and a ledger row written before the call and completed after it
- A send carries our own `Message-ID`. After an uncertain send the Backend looks for that id in Sent. Found means sent; not found means it cannot tell, and it asks the user through an approval item. A send is never retried blindly
- The 10 second undo window starts when the Backend receives the send. A send still in the engine's offline queue is cancelled there

## Events and triggers

- Triggers are reactive (an event happened) or proactive (a schedule came due)
- Events are rows written in the same transaction as the change that caused them. A matcher turns events into jobs for subscribed agents, deduplicated by agent and event
- Reactive triggers: new mail arrived, mail sorted into a category or moved to a folder, mail flagged, label or star added, another agent finished, approval resolved, calendar event approaching, file changed
- Trigger filters (sender, account, folder, agent) are evaluated in code before any LLM call. A proactive wakeup first runs a code-only check for anything new, so an idle wakeup costs no tokens
- Each event carries its cause and a hop count. A chain of agents waking agents stops at depth 3. An agent's actions may wake other agents, never itself
- A burst of mail creates one run per mail, with one run at a time per agent
- New mail is detected by provider push where offered (Gmail watch, Microsoft Graph subscriptions) and by polling jobs otherwise. No held IMAP IDLE connections. Polled accounts are checked every 60 seconds. Target: under 60 seconds from arrival to wakeup with push, under 120 with polling
- Provider webhooks arrive as JSON, the allowed exception to protobuf only. A webhook handler does nothing but enqueue a fetch job
- "Calendar event approaching" carries a lead time in seconds set on the trigger
- A watch folder is the folder filter on "moved to a folder": the agent wakes once for each mail that enters that folder, whoever put it there (the user, a rule or another agent). An agent may watch several folders. For Drive and OneDrive the same idea is "file changed" filtered to a folder
- "Another agent finished" takes an agent filter naming one of the user's own agents, system agents included. Without the filter it would fire after every agent, so the filter is required. The run receives that agent's deliverable as its handover
- A schedule is a fixed option or an interval in seconds, never under 900

## Agent runs

- A run is a loop, and each turn of it is a job. "Advance run" loads the transcript, makes one LLM call, stores the response and enqueues the tool calls; each tool result re-enqueues "advance run"
- A run stops at 30 steps or 600 seconds of active time
- The prompt is assembled from stored parts, stable first so most of each call is a cache hit: system prompt, tools, the agent's instructions, permissions, then notes, mail content and other context
- One fixed, versioned system prompt serves every agent. All variation lives in the user's instructions, the permissions and the tool list
- A run ends by calling a `finish` tool with a typed deliverable. Prose is never parsed: a reply with no tool call fails the run
- Three kinds of memory, kept apart: the transcript (within one run), notes (across runs), the handover (from one agent to the next: the typed deliverable only, never the transcript)
- Notes are written by the agent through a tool, size-capped, readable and editable by the user. One notebook per agent, plus one note per agent and thread
- Context for a thread is the latest messages in full plus a stored summary of the rest. Older messages and attachments are fetched on demand through tools
- An agent cannot create or change agents or schedules, its own included
- A dry run is the same executor with every write recorded instead of executed. Its LLM calls count toward the caps
- "Pause all" lets the current step finish, then moves each run to paused. Resuming returns them to ready

## Agent chat

The user can write to an agent and get an answer, for a question about their mail or calendar or a one-off request

- Chat exists for agents the user created and for Assistant. Gatekeeper, Security Guard, Scribe and News Bearer have no chat
- A chat turn is a run like any other. Its cause is a "user message" event, and its answer is the typed deliverable of `finish`: text plus references to the threads and events it is about. Prose is still never parsed
- One conversation per agent. It is a fourth kind of memory, kept apart from the other three: it holds the user's messages and the agent's answers, never a transcript. A turn's prompt gets the latest turns in full plus a stored summary of the rest
- The user's message is an instruction for that turn only. It cannot change the agent's stored instructions, permissions or schedule. Mail and notes read during the turn stay untrusted, and the run's read state applies as in any run
- The agent's permissions apply unchanged. An action that resolves to Ask parks the turn and appears in the approval queue and in the conversation
- One running turn at a time per conversation: a message sent while a turn is running is refused, and the engine receives the running turn as an explicit state. A turn parked on an approval does not hold the conversation: the user can keep writing, and the parked turn finishes when the approval is resolved
- The answer reaches the app through sync when the turn finishes. There is no token-by-token streaming at first
- Chat turns count toward the safety caps and the token budget. Without a key, chat is off, as an explicit state
- A conversation is kept 90 days, like the run log, and the user can clear it
- An agent that can chat may read what Scribe and News Bearer already wrote: stored thread summaries, and the latest briefing plus the four before it. It reads them through tools when a turn needs them, so they are not part of every prompt. They are deliverables, never transcripts, and they enter the prompt as delimited data because they are built from mail

## Permissions and untrusted content

- Permissions in the prompt are advice. Enforcement is code: the model is offered only tools that are not Off, and one permission function decides Auto or Ask when a tool call executes
- Mail content is hostile input. It is delimited as data in the prompt, and a run that has read any mail body or any note cannot Auto-send to anyone who is not already a participant of that thread, whatever its settings
- An action set to Ask parks the run, which resumes when the approval is resolved. An approval expires after 7 days and the run ends as declined. An edit made before approving is recorded in the transcript
- Notes can carry injected text into later runs, so they are treated as untrusted too

## LLM access and limits

- Bring your own key only, with one exception: embeddings use a platform key. Keys are encrypted at rest with a key kept outside the database
- Work that can wait (the briefing) may use a provider's batch API
- Fixed safety caps apply to everyone as loop protection: steps per run, runs per agent per day, concurrent runs. A user may also set a daily token budget; none is imposed
- At a cap, new runs stop being created, in-flight runs finish their step and park, and the user is told why. Mail sync, sending and manual actions keep working
- Without a key, AI compose, Scribe and the user's own agents are off, and the engine receives that as an explicit state. System agents run their code stage only

## Stored mail

- When the engine serves a body that is no longer cached, it asks the Backend, which fetches it from the provider
- Postgres holds metadata and thread structure for all mail, and extracted text for the last 90 days. Older bodies and all attachments are fetched from the provider on demand
- The Backend parses MIME once, at ingest. The Rust engine receives parsed parts and only sanitizes HTML when it serves a body for display

## Sign-in

- Users sign in with Google or Microsoft. An EveryAI account is added later
- Sign-in is separate from connecting mail accounts, though the first sign-in can connect that provider's mailbox in the same step

## Schema

- Each store has its own hand-written `.sql` migrations: Postgres holds the server's truth, SQLite the local replica, and their shapes differ. Protobuf is the only shared contract
- A generated check fails CI when a synced message and its tables drift apart

## Sync and live updates

- One server-to-engine stream per client: a single long-lived HTTP response carrying length-prefixed protobuf frames (the Connect server-streaming framing). It is Server-Sent Events in shape, without the text encoding that would force base64 onto protobuf
- Catch-up and live updates are the same call: the engine opens the stream with its cursor and receives every change after it. On reconnect it sends its last cursor
- A client with no cursor (a first launch, every web tab) first receives a snapshot of the newest mail, then the live changes. Older mail is paged in on demand by unary calls
- Everything from the engine to the Backend stays a unary POST
- The stream is fed by the events table that also drives triggers, so a change is recorded once
- Events are kept 30 days. A cursor older than the oldest kept event gets a resync frame, and the engine rebuilds its replica
- A heartbeat frame every 25 seconds keeps intermediaries from closing an idle stream
- The HTTP adapter must support a streamed response body, and so must Anole before the swap

## Embeddings

- pgvector serves four uses: retrieval over mail for agents, search by meaning for the user, sorting mail into categories, and recalling the user's own writing so drafts sound like them
- One hosted embedding model for every user, the Gemini embedding API with vectors shortened to 768 dimensions, paid by the platform, so there is one index and every user has these features whatever key they brought
- Vectors are kept for all mail. Text leaves after 90 days, its vector stays, and a hit on older mail fetches the body from the provider

## Providers

- Gmail through the Gmail API, Outlook through Microsoft Graph, everything else (iCloud, Yahoo, Fastmail, others) through IMAP and SMTP. Three mail connectors behind one interface
- Proton Mail is not supported: its Bridge runs on the user's machine, out of the Backend's reach
- Calendar: Google Calendar, Outlook calendar and CalDAV (iCloud, Fastmail) in one view, where the user can see, create, edit, delete and answer invitations, also from inside a thread
- Drive and OneDrive are for agents only: files and sheets an agent is pointed at. The app has no file browser

## System agents

Everything the app does with judgement is done by an agent the user can see. Nothing agentic is hidden in a feature

- Every new user gets system agents. They can be deactivated, never deleted; an agent the user created can be deleted
- On a system agent the user can set each action Off, Ask or Auto, change its schedule, and append instructions. Its built-in instructions are fixed and versioned, so an update reaches everyone
- A system agent may have a built-in code stage that runs first and costs nothing. Only what the code cannot decide goes to the model, and only when the user has a key. Without a key the system agents still work, less finely
- Gatekeeper sorts and flags every new mail. Its code stage reads bulk headers, the provider's spam verdict, SPF, DKIM and DMARC results, the block list and whether the user has written to the sender
- Gatekeeper sorts into EveryMail categories: For you, Updates and Reading, shown as tabs, plus folders the user creates. Nothing moves at the provider unless an agent or the user moves it
- When an account is first connected, Gatekeeper sweeps the last 90 days of mail already there with its code stage only: it sorts it, and for each bulk sender flooding the inbox it asks once whether to unsubscribe. The sweep makes no model call, so it costs no tokens and does not count against the caps. Sorting done by the sweep wakes no other agent
- A deactivated Gatekeeper sorts and flags nothing: all mail lands in For you and Security Guard never wakes. The block list and the user's rules still apply, because they are code
- Security Guard wakes on mail flagged as malicious or a scam. On its own it moves the mail to Quarantine, adds the sender to the block list, reports it to the provider as spam, and notifies the user of what it caught. It never follows a link and never unsubscribes
- Quarantine is an EveryMail folder. Reporting spam moves the mail to the provider's own spam folder, where the provider's retention applies; EveryMail drops its copy 30 days after the mail entered Quarantine
- Scribe writes summaries and reply suggestions. It runs the first time one is needed, when the user opens a thread or another agent needs context, never on arrival
- News Bearer writes a briefing at its set times: what agents did, what waits, newsletters and updates condensed, the day's calendar. It appears as a view with a notification, and a permission lets it also mail the briefing. Outside news comes later, see below
- News topics, once EveryAI's web search tool is wired in: the user keeps a list of topics on News Bearer, at most 10, and each briefing gains a news section with a few items per topic, each with its source link. Web results are untrusted content like mail. With no topics the briefing has no news section and makes no search
- Assistant answers the user in chat about their mail and calendar across every account. It has no trigger and no code stage: it wakes only on a message, so it is off without a key. It reads and searches on its own, and any write it proposes follows its Off, Ask or Auto settings like every agent
- Order on a new mail: rules, then Gatekeeper. Security Guard wakes on "flagged". Other agents trigger on "sorted into a category or folder", or on "new mail", which fires when Gatekeeper has finished, whatever the outcome. With Gatekeeper deactivated, "new mail" fires at once
- Gatekeeper and Security Guard never use a batch API: sorting cannot wait

## Rules and the block list

- Rules are a fixed set of conditions and actions evaluated in code on every new mail. Anything that needs judgement is an agent, not a rule
- The block list is enforced in code: mail from a blocked sender goes to Quarantine without any agent running

## Notifications

- One notification job kind feeds every channel. Desktop: the engine raises a system notification from the sync stream while the app runs. Web: Web Push. Phones: APNs and FCM in the Tauri mobile apps
- Notified: something waits for approval, Security Guard caught something, a briefing is ready, and mail an agent was told to report at once

## Safety caps and snooze presets

- Fixed caps: 30 steps per run, 200 runs per agent per day, 3 concurrent runs per user
- Snooze presets, in the user's zone: Later today is 18:00, or 3 hours from now once it is past 15:00; Tomorrow is 08:00; This weekend is Saturday 09:00; Next week is Monday 08:00

## Running on the VM

- Bun as a systemd service and Postgres from the distribution's packages. No containers in production
- TLS ends at the reverse proxy that already serves the EveryAI services. It must not buffer the sync stream
- The domain is mail.everyai.dev, shared by the landing page, the web app and the Backend's calls
- The key that encrypts provider tokens and LLM keys is a file on the VM, outside Postgres, readable only by the account the Backend service runs as
- No backups at first. Mail can be synced again from the providers; agents, notes, rules and run logs would be lost with the VM. This is acceptable only before real users and is revisited then
- Logs go to the system journal and carry ids, kinds and durations only: never mail content, prompts or tokens
- A watchdog job checks queue depth, the oldest ready job and failure rates, and notifies the owner when a threshold is crossed. No external monitoring service

## Gmail access

- Gmail runs in Google's testing mode, limited to 100 named test users, while the restricted-scope verification and its security assessment proceed in parallel. Microsoft and IMAP accounts are open to everyone from the start
- Until that verification is done, Send cannot be set to Auto on a Gmail account: agents prepare replies as drafts in EveryMail and the user approves each one

## User data

- Deleting an account, or disconnecting one mailbox, purges at once: jobs are cancelled, provider tokens revoked, and that data deleted from Postgres. Nothing changes at the provider
- A user can export what only EveryMail holds, as one file: agents and their instructions, notes, rules, the block list and the run log. Mail is not exported; the provider already offers that

## Pricing

- Free, with bring your own key. A paid tier that includes model usage comes later, so token metering is built in a way a plan can attach to. No billing code now

## Landing page

- An Astro site in this repo, served at the root of mail.everyai.dev, with the waitlist form from the design
- The web app lives on the same origin under Gmail-style paths: `/u/<n>/` followed by the view and, for a thread, its id. `<n>` is the slot of the signed-in user in this browser, 0 until several sign-ins are supported
- The Backend's calls stay on the same origin under their `/api.v1.` paths, so the session cookie and same-origin rule need no cross-origin setup
- On web the path is the engine's navigation state written out: the engine produces it, and opening a link is an intent sent to the engine. The Frontend does no routing of its own
- The waitlist form posts to one public Backend call. This is the single exception to the engine being the Backend's only client
