# EveryMail

Mail client that connects every inbox, plus drive, OneDrive and calendar with read/write access. Users define agent profiles and triggers; agents run the defined tasks autonomously on a schedule.

Design source: the EveryMail artifact in the owner's Claude account (https://claude.ai/artifact/7p4rWpG1SMMspPD8EJYrp3), built on the EveryAI Design System.

## Goals

- Minimal tech debt
- Code as thin as possible
- Desktop app stays under 300-400 MB RAM on any desktop

## Terms

- **Backend**: whatever runs on the VM. Currently Bun + PostgreSQL
- **Rust engine**: the Rust crate that ships with the client (native in Tauri, WASM on web). It acts as a pseudo backend and middleware for the UI layer
- **Frontend**: explicitly the part users interact with and see: the React UI

## Stack

| Part                                     | Choice                                                                                                                                                                               | Status                                              |
| ---------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------- |
| Backend                                  | Bun (`Bun.serve`), everything in TypeScript: agents, scheduling, notifications, provider/connector logic via TypeScript libraries                                                    | Chosen                                              |
| Backend database access                  | Bun's built-in Postgres client (`Bun.sql`) with plain `.sql` migrations. No ORM                                                                                                      | Chosen                                              |
| Engine local database                    | rusqlite, bundled, added with the first local table                                                                                                                                  | Chosen                                              |
| Protobuf transport                       | Plain protobuf over HTTP POST: `POST /api.v1.Service/Method`, body `application/proto` (the Connect unary wire shape, no RPC framework). Streaming is decided with the sync protocol | Chosen                                              |
| Code generation                          | buf with protobuf-es for TypeScript, prost for Rust. Generated code is not committed                                                                                                 | Chosen                                              |
| Monorepo tooling                         | Bun workspaces and one Cargo workspace. Biome for TypeScript, clippy and rustfmt for Rust                                                                                            | Chosen                                              |
| Backend database                         | PostgreSQL with pgvector                                                                                                                                                             | Chosen, gated on fitting under 100 MB on the GCE VM |
| Rust engine                              | Rust: all heavy tasks and async threads, for speed and performance                                                                                                                   | Chosen                                              |
| Frontend                                 | React + Vite                                                                                                                                                                         | Chosen                                              |
| Desktop shell                            | Tauri 2, hosting the Frontend and the Rust engine                                                                                                                                    | Chosen                                              |
| Hosting                                  | GCE, alongside the EveryAI services                                                                                                                                                  | Chosen                                              |
| Mobile, job scheduling, auth, LLM access |                                                                                                                                                                                      | Undecided                                           |

No Rust in the Backend. Rust lives only in the Rust engine

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
- Performant and low RAM: the whole desktop app (Frontend plus Rust engine) must not exceed 300-400 MB on any desktop

## Rust engine

The heavy lifter, and a pseudo backend for the Frontend. It exists to lower the memory profile and raise performance

- Every heavy, performance-critical or thread-blocking process runs here: parsing, sanitizing, indexing, search, sync, local database access, crypto and the like
- One Rust crate for all heavy work. Desktop runs it natively in Tauri; web runs the same crate compiled to WASM in a Web Worker. No second implementation in TypeScript
- Keeps a local SQLite database on desktop, so lists, search and reading are served locally. It is synced from the Backend over protobuf

## Engine boundaries

The Rust engine sits between the Frontend and the Backend: the Frontend talks only to the engine, and the engine is the only client of the Backend

- The engine owns the Backend connection, the session and its credentials, sync, the local database, the offline queue and shaping data for the Frontend
- Engine to Backend: protobuf over the Backend's protobuf transport (package `api.v1`)
- Frontend to engine: protobuf bytes through Tauri's raw binary IPC, with Tauri channels for streaming updates. On web, the Worker running the WASM engine passes the same bytes to the main thread
- Frontend messages are shaped for the Frontend (package `ui.v1`, in the same `proto/` tree): view messages such as rows and thread views going out, small intent messages coming in. They import shared domain messages and add only presentation fields
- Anything with states is a `oneof` sent by the engine (for example loading, loaded, failed), so TypeScript never derives state from flags
- Message bodies and attachments are served by the engine through a Tauri custom URI scheme into a sandboxed iframe or image, so they never enter the JavaScript heap. This is the one explicit exception to protobuf-only: raw content bytes, not protobuf and not JSON
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

## Open questions

- Web build: where the web app keeps its local replica (for example SQLite compiled to WASM on the browser's origin private file system). The browser transport is decided: fetch, same origin as the Backend
- Web build: how message bodies and attachments are served, since the custom URI scheme exists only in Tauri (for example a Service Worker in front of the engine Worker)
- Which side parses MIME and sanitizes HTML. The Backend needs parsed mail for agents and the engine is assigned parsing: doing both duplicates logic, so one side owns it and the other receives protobuf
- Streaming transport for sync, decided together with the sync protocol
- Sync protocol between the Backend (Postgres) and the Rust engine's local SQLite replica
- Schema ownership: Postgres tables, SQLite tables and protobuf messages describe the same entities. Decide the single source and generate the rest, so the schema is not written three times
- Web engine: which context owns the local database when several tabs are open
