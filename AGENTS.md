# EveryMail

Mail client that connects every inbox, plus drive, OneDrive and calendar with read/write access. Users define agent profiles and triggers; agents run the defined tasks autonomously on a schedule.

Design source: the EveryMail artifact in the owner's Claude account (https://claude.ai/artifact/7p4rWpG1SMMspPD8EJYrp3), built on the EveryAI Design System.

## Where the detail lives

This file holds the rules. How each part works is in the docs, what to build is in the roadmaps, and what is still undecided is a "Decide:" task in a roadmap, nowhere else

| Topic                                           | How it works                                                                                                                                                                                             | What to build                                                                                                                                                                    |
| ----------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Jobs, the executor, leases, lanes               | [Jobs](docs/BACKEND.md#jobs)                                                                                                                                                                             | [Jobs, time and side effects](TODO-backend.md#jobs-time-and-side-effects)                                                                                                        |
| Server time, schedules, snooze, undo send       | [Time](docs/BACKEND.md#time), [Safety caps and snooze presets](docs/BACKEND.md#safety-caps-and-snooze-presets)                                                                                           | [Jobs, time and side effects](TODO-backend.md#jobs-time-and-side-effects), [Write actions](TODO-backend.md#write-actions)                                                        |
| Idempotency and sending mail                    | [Side effects](docs/BACKEND.md#side-effects)                                                                                                                                                             | [Write actions](TODO-backend.md#write-actions)                                                                                                                                   |
| Triggers and events                             | [Events and triggers](docs/BACKEND.md#events-and-triggers)                                                                                                                                               | [Agents and approvals](TODO-backend.md#agents-and-approvals)                                                                                                                     |
| Agent runs, prompts, memory                     | [Agent runs](docs/BACKEND.md#agent-runs)                                                                                                                                                                 | [Agent runs](TODO-backend.md#agent-runs)                                                                                                                                         |
| Permissions and hostile mail content            | [Permissions and untrusted content](docs/BACKEND.md#permissions-and-untrusted-content)                                                                                                                   | [Agent runs](TODO-backend.md#agent-runs)                                                                                                                                         |
| Gatekeeper, Security Guard, Scribe, News Bearer | [System agents](docs/BACKEND.md#system-agents), [Rules and the block list](docs/BACKEND.md#rules-and-the-block-list)                                                                                     | [System agents](TODO-backend.md#system-agents)                                                                                                                                   |
| LLM keys, caps, embeddings                      | [LLM access and limits](docs/BACKEND.md#llm-access-and-limits), [Embeddings](docs/BACKEND.md#embeddings)                                                                                                 | [LLM access and limits](TODO-backend.md#llm-access-and-limits), [Embeddings](TODO-backend.md#embeddings)                                                                         |
| Mail providers, calendar, stored mail           | [Providers](docs/BACKEND.md#providers), [Stored mail](docs/BACKEND.md#stored-mail), [Gmail access](docs/BACKEND.md#gmail-access)                                                                         | [First account connected](TODO-backend.md#first-account-connected-and-mail-synced-read-only), [Calendar](TODO-backend.md#calendar)                                               |
| Sync between Backend and engine                 | [Sync and live updates](docs/BACKEND.md#sync-and-live-updates), [Schema](docs/BACKEND.md#schema)                                                                                                         | [First account connected](TODO-backend.md#first-account-connected-and-mail-synced-read-only), [engine side](TODO-rust-core.md#first-account-connected-and-mail-synced-read-only) |
| Sign-in and sessions                            | [Sign-in](docs/BACKEND.md#sign-in), [Session](docs/ENGINE.md#session)                                                                                                                                    | [Foundations](TODO-backend.md#foundations), [Session and local database](TODO-rust-core.md#session-and-local-database)                                                           |
| Notifications                                   | [Notifications](docs/BACKEND.md#notifications)                                                                                                                                                           | [Backend](TODO-backend.md#notifications), [engine](TODO-rust-core.md#notifications)                                                                                              |
| Deployment, HTTP engine, Postgres, logs         | [Deployment](docs/BACKEND.md#deployment), [Backend engine](docs/BACKEND.md#backend-engine), [Backend database](docs/BACKEND.md#backend-database), [Running on the VM](docs/BACKEND.md#running-on-the-vm) | [Deployment and measurement](TODO-backend.md#deployment-and-measurement), [Anole swap](TODO-backend.md#anole-swap)                                                               |
| Deletion, export, pricing, landing page         | [User data](docs/BACKEND.md#user-data), [Pricing](docs/BACKEND.md#pricing), [Landing page](docs/BACKEND.md#landing-page)                                                                                 | [User data](TODO-backend.md#user-data), [Waitlist](TODO-backend.md#waitlist), [Landing page](TODO-frontend.md#landing-page)                                                      |
| Local replica, search, drafts                   | [Local replica](docs/ENGINE.md#local-replica), [Drafts](docs/ENGINE.md#drafts)                                                                                                                           | [Session and local database](TODO-rust-core.md#session-and-local-database), [Search](TODO-rust-core.md#search)                                                                   |
| Message bodies and their height                 | [Message bodies on web](docs/ENGINE.md#message-bodies-on-web), [Body height](docs/ENGINE.md#body-height)                                                                                                 | [Inbox and thread reading](TODO-rust-core.md#inbox-and-thread-reading), [Web parity](TODO-rust-core.md#web-parity)                                                               |
| Desktop shell, stores, mobile                   | [Desktop shell](docs/ENGINE.md#desktop-shell), [Mobile](docs/ENGINE.md#mobile)                                                                                                                           | [Packaging and measurement](TODO-rust-core.md#packaging-and-measurement), [Packaging and measurement gates](TODO-frontend.md#packaging-and-measurement-gates)                    |
| Screens and motion                              | The UI/UX rules below                                                                                                                                                                                    | [TODO-frontend.md](TODO-frontend.md)                                                                                                                                             |

A change to how something works is made in the doc and its roadmap together. A rule in this file wins over both

## Goals

- Minimal tech debt
- Code as thin as possible
- Desktop app stays under 300-400 MB RAM on any desktop

## Terms

- **Backend**: whatever runs on the VM. Currently Bun + PostgreSQL
- **Rust engine**: the Rust crate that ships with the client (native in Tauri, WASM on web). It acts as a pseudo backend and middleware for the UI layer
- **Frontend**: explicitly the part users interact with and see: the React UI

## Stack

| Part                    | Choice                                                                                                                                                                                                                      | Status                                              |
| ----------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------- |
| Backend                 | Bun (`Bun.serve`), everything in TypeScript: agents, scheduling, notifications, provider/connector logic via TypeScript libraries                                                                                           | Chosen                                              |
| Backend database access | Bun's built-in Postgres client (`Bun.sql`) with plain `.sql` migrations. No ORM                                                                                                                                             | Chosen                                              |
| Engine local database   | rusqlite, bundled, added with the first local table                                                                                                                                                                         | Chosen                                              |
| Protobuf transport      | Plain protobuf over HTTP POST: `POST /api.v1.Service/Method`, body `application/proto` (the Connect unary wire shape, no RPC framework). Sync adds one streamed response, see Sync transport                                | Chosen                                              |
| Code generation         | buf with protobuf-es for TypeScript, prost for Rust. Generated code is not committed                                                                                                                                        | Chosen                                              |
| Monorepo tooling        | Bun workspaces and one Cargo workspace. Biome for TypeScript, clippy and rustfmt for Rust                                                                                                                                   | Chosen                                              |
| Backend database        | PostgreSQL with pgvector                                                                                                                                                                                                    | Chosen, gated on fitting under 100 MB on the GCE VM |
| Rust engine             | Rust: all heavy tasks and async threads, for speed and performance                                                                                                                                                          | Chosen                                              |
| Frontend                | React + Vite                                                                                                                                                                                                                | Chosen                                              |
| Desktop shell           | Tauri 2, hosting the Frontend and the Rust engine                                                                                                                                                                           | Chosen                                              |
| Hosting                 | GCE, alongside the EveryAI services                                                                                                                                                                                         | Chosen                                              |
| Job scheduling          | One jobs table in Postgres, claimed in batches with `FOR UPDATE SKIP LOCKED`, inside the Backend process                                                                                                                    | Chosen                                              |
| LLM access              | Bring your own key only, keys stored encrypted on the Backend. Anthropic, OpenAI, Google Gemini and OpenRouter first; any dedicated provider can be added. Embeddings are the exception: one hosted model on a platform key | Chosen                                              |
| Sign-in                 | Google or Microsoft now, an EveryAI account later                                                                                                                                                                           | Chosen                                              |
| Sync transport          | One streamed HTTP response of length-prefixed protobuf frames from Backend to engine; unary POST the other way                                                                                                              | Chosen                                              |
| Mobile                  | Tauri 2 mobile, same Frontend and same Rust crate                                                                                                                                                                           | Chosen                                              |

No Rust in the Backend. Rust lives only in the Rust engine

## Hard requirements (whole app)

1. NO IMPLICIT STATE MACHINES
2. ANY STATE THAT IS NOT EXPLICITLY DEFINED IS NOT PART OF THE CODEBASE, on Backend, Rust engine, Frontend or anywhere else
3. CODE DUPLICATION IS STRICTLY FORBIDDEN
4. THE IMPLEMENTATION SHOULD BE THE SIMPLEST POSSIBLE IMPLEMENTATION OF A PARTICULAR TASK DEFINED

## Frontend

The part users interact with and see. React + Vite, TypeScript for developer experience and its ecosystem

- TypeScript renders and handles input only. It holds no business logic and keeps in memory only what is on screen; the Rust engine owns the data
- One Frontend codebase: the web app and the Tauri desktop app share the same React UI
- The Frontend talks only to the Rust engine, never to the Backend
- Libraries that are already optimized and save real work are welcome, loaded lean: import only the parts used. Motion is the animation library for shared layout, exits and gestures; plain CSS covers presses, fades and dots
- Performant and low RAM: the whole desktop app (Frontend plus Rust engine) must not exceed 300-400 MB on any desktop

## Rust engine

The heavy lifter, and a pseudo backend for the Frontend. It exists to lower the memory profile and raise performance

- Every heavy, performance-critical or thread-blocking process runs here: sanitizing, indexing, search, sync, local database access, crypto and the like
- One Rust crate for all heavy work. Desktop runs it natively in Tauri; web runs the same crate compiled to WASM in a Web Worker. No second implementation in TypeScript
- Keeps a local SQLite database, so lists, search and reading are served locally. It is synced from the Backend over protobuf

## Engine boundaries

The Rust engine sits between the Frontend and the Backend: the Frontend talks only to the engine, and the engine is the only client of the Backend. The one exception is the landing page's waitlist form, which posts to one public Backend call

- The engine owns the Backend connection, the session and its credentials, sync, the local database, the offline queue and shaping data for the Frontend
- Engine to Backend: protobuf over the Backend's protobuf transport (package `api.v1`)
- Frontend to engine: protobuf bytes through Tauri's raw binary IPC, with Tauri channels for streaming updates. On web, the Worker running the WASM engine passes the same bytes to the main thread
- Frontend messages are shaped for the Frontend (package `ui.v1`, in the same `proto/` tree): view messages such as rows and thread views going out, small intent messages coming in. They import shared domain messages and add only presentation fields
- Anything with states is a `oneof` sent by the engine (for example loading, loaded, failed), so TypeScript never derives state from flags
- Message bodies and attachments are served by the engine through a Tauri custom URI scheme into a sandboxed iframe or image, so they never enter the JavaScript heap. On web a Service Worker in front of the engine Worker plays the part of the scheme. This is the one explicit exception to protobuf-only: raw content bytes, not protobuf and not JSON
- The engine does transport, sync, caching, queueing and view shaping only. It makes no domain decision the Backend also makes, such as agent permissions, to avoid duplicating Backend logic
- Platform differences (transport, storage and the async runtime: native networking, SQLite and threads on desktop; browser fetch and streams, SQLite on WASM and a single Worker on web) sit behind a small platform adapter layer inside the crate, and nothing above it is aware of the target

## State ownership

The Rust engine owns all application state. TypeScript owns only presentation state. This is server-side rendering of state, not of markup: the engine decides what is shown and in what state, the Frontend decides how it looks and moves

| Rust engine owns (survives relaunch, affects which data is fetched, or matters to another screen) | TypeScript owns (changes how a pixel looks this second) |
| ------------------------------------------------------------------------------------------------- | ------------------------------------------------------- |
| Accounts, mail, threads, agents, approvals, settings                                              | Hover, press, focus ring                                |
| Current screen and navigation                                                                     | Popover open or closing phase                           |
| Selected thread, search query and results, filters                                                | Animation progress and measured layout                  |
| Draft contents (persisted)                                                                        | Keystrokes before they are committed                    |
| Sync and connection status                                                                        | Pixel scroll offset                                     |
| Theme and density preference                                                                      | Which row is mid-animation                              |

- The engine sends screen-level view models, not layout trees and not widget-level state. The protobuf never describes markup
- Composer text lives in TypeScript while typing and is sent to the engine on a debounce; the engine owns the saved draft
- Lists: the engine owns the scroll anchor (a row id). TypeScript reports the visible range and the engine returns that window, with overscan so scrolling never waits. Rows carry stable ids so exits and `layoutId` reuse animate correctly
- Presentation state in TypeScript is an allowlist: each item is declared with its reason (it must update within a frame, or it is input before commit) and is an explicit enum, never a pair of booleans. Any other TypeScript state is a bug and belongs in the engine

## Contract

- Protobuf is the single source of truth for every type shared by the Backend and the Rust engine
- Protobuf only on the wire: no JSON anywhere unless explicitly needed (for example an external provider API that requires it)

## UI/UX: minimize and justify

Every use of the following needs a stated reason:

1. Tags and pills
2. Excessive italics
3. Instrument Serif
4. Negative kerning
5. Over-the-top detailed UI
6. Overusing dividers
7. Color shades and opacities
8. Containers with borders
9. Overly detailed diagrams
10. Verbose, redundant text
11. Code breadcrumbs
12. Section numbering
13. Eyebrows
14. Dithering
15. ASCII images
16. Squircles
17. Excessive mono fonts
18. ALL CAPS subtitles
19. Gradient backgrounds
20. Periods in titles

## UI/UX: always

1. Ease-out for entrances
2. Dropdowns at 150-250ms
3. Stagger list rows by 30ms
4. Scale to 0.97 on press
5. Stretch moving knobs 12%
6. Stagger as details settle
7. Arcs for object movement
8. Pulse to confirm success
9. Ease-in-out for moves
10. Start popovers at 0.95
11. Enter from the trigger
12. Exit to the trigger
13. No bounce on everyday UI
14. Don't tween while held
15. Make exits 20% shorter
16. Overlap old and new states
17. Masks for reveals and colour
18. Crossfade icons + blur
19. Rubber band past bounds
20. Reuse the same layoutId

## Tests

- Test only HARD and DETERMINISTIC logic
- Remove one-time tests and tests that existed only to debug something once
- Think twice before adding a test that depends on how fast the machine is
- A CI test that turns out flaky is deleted AND rewritten from scratch
