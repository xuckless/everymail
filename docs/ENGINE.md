# Rust engine

How the Rust engine works beyond the rules in [AGENTS.md](../AGENTS.md), which holds its boundaries and what state it owns. The tasks that build this are in [TODO-rust-core.md](../TODO-rust-core.md), and what is still undecided is there as "Decide:" tasks.

## Local replica

- Desktop and mobile: SQLite on disk. It holds metadata and thread structure for all mail, and body text for the last 90 days, the same window as the Backend. Older bodies come through the Backend on demand
- Web: SQLite in memory, one per tab, filled from the Backend on load. Nothing persists in the browser and tabs do not coordinate. The web app needs a connection, and has no offline queue: an action taken without a connection fails at once
- The database is not encrypted; the operating system's disk encryption covers it. No secret is stored in it
- Local search is SQLite FTS5, updated in the same transaction as the mail row. Search by meaning comes from the Backend

## Session

- Desktop and mobile keep the session credential in the operating system's keychain
- On a Linux setup with no keychain service the credential goes into a file only the user can read, and the app says so once
- On web the Backend sets an HttpOnly cookie, so the credential never enters JavaScript or WASM memory

## Drafts

- A draft is saved locally at once and synced through the Backend, so it appears on the user's other devices and agents can see it. It is not written to the provider's Drafts folder

## Message bodies on web

- A Service Worker answers body and attachment URLs by asking the engine Worker for the bytes. The iframe loads an ordinary same-origin URL, as it does on desktop through the custom URI scheme, and the Frontend code is the same on both

## Desktop shell

- Tauri commands are denied by default: an app manifest names each command the webview may call
- Windows and macOS ship through the Microsoft Store and the Mac App Store, which also deliver updates. Linux ships from the first release through an install script fetched with curl, which also updates

## Body height

- The engine appends one small trusted script to each served body, allowed by a per-response nonce, that reports the body's height to the page. Every other script stays blocked by the sanitizer and the content policy

## Mobile

- Tauri 2 mobile: the same Frontend and the same crate, compiled natively for iOS and Android. No second interface
