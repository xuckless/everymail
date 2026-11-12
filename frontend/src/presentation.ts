// The presentation-state allowlist (AGENTS.md, State ownership). This module is the only place
// TypeScript holds state: every React state hook and every mutable variable of the Frontend is
// declared here, each with its reason, and each is an explicit enum or tagged union.
//
// Reasons, the only two that qualify:
//   Frame: it must update within a frame, so a round trip through the engine is too slow.
//   Input: it is input before the user commits it.
// Anything else belongs in the engine: add an intent and a view field instead.
//
// To add an entry: declare its type here as a union of string literals or a tagged union (never a
// pair of booleans), write its reason in the comment above it, and export one hook or factory.
// Screens import the hook; they never call useState, useReducer or useSyncExternalStore themselves.

import {
	Component,
	type ReactNode,
	useCallback,
	useEffect,
	useReducer,
	useRef,
	useState,
	useSyncExternalStore,
} from "react";

// Frame: a popover, menu or sheet the user opens and closes. The engine does not know it is open;
// "opening" and "closing" are the phases its enter and exit animations run in.
export type OverlayPhase = "closed" | "opening" | "open" | "closing";
// toggle: the trigger. dismiss: outside press, Escape, a chosen item. settled: an animation ended.
export type OverlayEvent = "toggle" | "dismiss" | "settled";

export function overlayStep(
	phase: OverlayPhase,
	event: OverlayEvent,
): OverlayPhase {
	switch (phase) {
		case "closed":
			return event === "toggle" ? "opening" : "closed";
		case "opening":
			if (event === "settled") return "open";
			return "closing";
		case "open":
			return event === "settled" ? "open" : "closing";
		case "closing":
			if (event === "settled") return "closed";
			return event === "toggle" ? "opening" : "closing";
	}
}

export function useOverlay() {
	const [phase, dispatch] = useReducer(overlayStep, "closed");
	return { phase, dispatch };
}

// Input: what the user has typed or picked in a form before it is committed (by Enter, Send, Save
// or a debounce). `initial` is the engine's value, such as a default port or a saved title; a form
// of several values (an event's fields) keeps each value in its own draft.
export function useDraft<Value = string>(
	...initial: Value extends string ? [Value?] : [Value]
): [Value, (next: Value) => void] {
	return useState((initial[0] ?? "") as Value);
}

// Input: text typed into a field that has a commit intent sent on a debounce (a draft, an agent's
// instructions, the builder's text). The pending send is input the engine has not taken yet: it
// waits `ms` after the last keystroke, is replaced by the next one, and is sent at once by `flush`
// (before an intent that must see it, such as Send) or when the field goes away.
export type Debounce =
	| { kind: "idle" }
	| { kind: "pending"; timer: number; run: () => void };

export function useDebounce(ms = 400) {
	const pending = useRef<Debounce>({ kind: "idle" });
	const flush = useCallback(() => {
		const now = pending.current;
		if (now.kind === "idle") return;
		clearTimeout(now.timer);
		pending.current = { kind: "idle" };
		now.run();
	}, []);
	useEffect(() => flush, [flush]);
	const schedule = useCallback(
		(run: () => void) => {
			const now = pending.current;
			if (now.kind === "pending") clearTimeout(now.timer);
			const timer = window.setTimeout(() => {
				pending.current = { kind: "idle" };
				run();
			}, ms);
			pending.current = { kind: "pending", timer, run };
		},
		[ms],
	);
	return { schedule, flush };
}

// Frame: the measured viewport class. Below 820px the phone layout replaces the sidebar; below
// 1180px the desktop drops the reading pane and the thread's reply panel. The only breakpoints.
// It also decides which intent a list row sends (Inbox, Quarantine): with a reading pane beside
// the list a click selects the row, without one it opens it. That is input handling, not state:
// the same click means what the visible layout shows it will do, and the engine owns the result.
export type Layout = "phone" | "list" | "split";

const phoneQuery = "(max-width: 819.98px)";
const listQuery = "(max-width: 1179.98px)";

function layoutNow(): Layout {
	if (matchMedia(phoneQuery).matches) return "phone";
	if (matchMedia(listQuery).matches) return "list";
	return "split";
}

function onLayoutChange(changed: () => void) {
	const queries = [matchMedia(phoneQuery), matchMedia(listQuery)];
	for (const query of queries) query.addEventListener("change", changed);
	return () => {
		for (const query of queries) query.removeEventListener("change", changed);
	};
}

export function useLayout(): Layout {
	return useSyncExternalStore(onLayoutChange, layoutNow);
}

// Frame: the user's last press, the point what it opens grows from (motion rules 11 and 12): the
// composer, and the theme's circular reveal (design/README.md). The engine's answer arrives a
// frame after the press; the first to take the press uses it. A key press clears it, so what a key
// opens (⌘N, Enter) grows from its own place, and a change the user did not just press for (the
// first view, another device) has none.
export type Press = { kind: "none" } | { kind: "at"; x: number; y: number };
let press: Press = { kind: "none" };

export function watchPresses() {
	const at = (event: PointerEvent) => {
		press = { kind: "at", x: event.clientX, y: event.clientY };
	};
	const key = () => {
		press = { kind: "none" };
	};
	addEventListener("pointerdown", at, true);
	addEventListener("keydown", key, true);
}

export function takePress(): Press {
	const now = press;
	press = { kind: "none" };
	return now;
}

// The engine's theme and density on the document root, where the design package's CSS reads them:
// written from the view, never read back. A theme the user pressed for is revealed with View
// Transitions where they exist, from the press; elsewhere the colours crossfade (`theme-fade`)
// until the body's colour transitions end.
export function showTheme(
	theme: "dark" | "light" | "system",
	timing: { duration: number; easing: string },
) {
	const root = document.documentElement;
	const apply = () => {
		if (theme === "system") delete root.dataset.theme;
		else root.dataset.theme = theme;
	};
	const now = takePress();
	switch (now.kind) {
		case "none":
			apply();
			return;
		case "at":
			break;
	}
	if (!document.startViewTransition) {
		root.classList.add("theme-fade");
		apply();
		requestAnimationFrame(() => {
			Promise.allSettled(
				document.body.getAnimations().map((animation) => animation.finished),
			).then(() => root.classList.remove("theme-fade"));
		});
		return;
	}
	const at = `${now.x}px ${now.y}px`;
	document
		.startViewTransition(apply)
		.ready.then(() =>
			root.animate(
				{ clipPath: [`circle(0px at ${at})`, `circle(160% at ${at})`] },
				{ ...timing, pseudoElement: "::view-transition-new(root)" },
			),
		);
}

export function showDensity(density: "compact" | "comfortable" | "roomy") {
	document.documentElement.dataset.density = density;
}

// Not presentation state, and the one exception: React reports a render error only through the
// state of an error boundary. It is terminal; the app shows the error and stops rendering views.
type Rendering = { kind: "rendering" } | { kind: "failed"; message: string };

export class RenderBoundary extends Component<
	{ children: ReactNode; failed: (message: string) => ReactNode },
	Rendering
> {
	state: Rendering = { kind: "rendering" };

	static getDerivedStateFromError(error: unknown): Rendering {
		return { kind: "failed", message: String(error) };
	}

	render() {
		switch (this.state.kind) {
			case "rendering":
				return this.props.children;
			case "failed":
				return this.props.failed(this.state.message);
		}
	}
}
