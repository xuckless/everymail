# Rust engine

How the Rust engine works beyond the rules in [AGENTS.md](../AGENTS.md), which holds its boundaries and what state it owns. The tasks that build this are in [TODO-rust-core.md](../TODO-rust-core.md), and what is still undecided is there as "Decide:" tasks.

## Local replica

- Desktop and mobile: SQLite on disk. It holds metadata and thread structure for all mail, and body text for the last 90 days, the same window as the Backend. Older bodies come through the Backend on demand
- Web: SQLite in memory, one per tab, filled from the Backend on load. Nothing persists in the browser and tabs do not coordinate. The web app needs a connection, and has no offline queue: an action taken without a connection fails at once
- The database is not encrypted; the operating system's disk encryption covers it. No secret is stored in it
- Local search is SQLite FTS5, updated in the same transaction as the mail row. Search by meaning comes from the Backend

## Backend port and the demo Backend

- Everything the Backend does for the engine goes through one trait, the Backend port. The engine applies a local change to its SQLite replica in one transaction and reports it; work only the Backend can do (sending, agent runs, model output, sync, permission and cap decisions, due-time jobs such as snooze and scheduled sends) is requested through the port and comes back as updates
- Until the real Backend exists, a demo implementation in `engine/src/demo/` plays it with scripted behaviour and a sample world delivered after sign-in. The hosts inject it; nothing outside that module knows it exists. It is replaced by the real Backend, not extended
- Short-lived results of an action are one mechanism: outcomes (done or failed, per target) and one notice (undo or info), both sent in the shell view
- What is open, expanded or being entered is engine state where it mixes with data (forms, replaced keys, expanded runs), and each typed field carries a revision that tells the Frontend when to drop what was typed

## Session

- Desktop and mobile keep the session credential in the operating system's keychain
- On a Linux setup with no keychain service the credential goes into a file only the user can read, and the app says so once
- On web the Backend sets an HttpOnly cookie, so the credential never enters JavaScript or WASM memory

## Drafts

- A draft is saved locally at once and synced through the Backend, so it appears on the user's other devices and agents can see it. It is not written to the provider's Drafts folder

## Message bodies on web

- A Service Worker answers body and attachment URLs by asking the engine Worker for the bytes. The iframe loads an ordinary same-origin URL, as it does on desktop through the custom URI scheme, and the Frontend code is the same on both
- On web the iframe carries no `sandbox` attribute, because Chromium does not send a sandboxed iframe's load through a Service Worker. The Service Worker's response carries `Content-Security-Policy: sandbox allow-scripts` instead, which gives the body the same opaque origin with scripts only. The engine supplies that header value; desktop keeps the attribute and sends the header too
- The body URL carries the engine's theme as a query part, so a theme change reloads each body. The Service Worker passes everything after `/body/`, query included, to the engine

## Desktop shell

- Tauri commands are denied by default: an app manifest names each command the webview may call
- Windows and macOS ship through the Microsoft Store and the Mac App Store, which also deliver updates. Linux ships from the first release through an install script fetched with curl, which also updates. The script checks a GPG signature on the release's SHA256SUMS file, then the hash, before it installs anything

## Body height

- The engine appends one small trusted script to each served body, allowed by a per-response nonce, that reports the body's height to the page. Every other script stays blocked by the sanitizer and the content policy
- The message is a bare number of CSS pixels: the body element's height, posted on load and on every resize. The page accepts it only from the iframe's own window

## Mobile

- Tauri 2 mobile: the same Frontend and the same crate, compiled natively for iOS and Android. No second interface
- The phone apps go out through TestFlight and Play testing first, and to the public stores once the product is past alpha
