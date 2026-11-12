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
	useReducer,
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

// Input: text typed into a field before it is committed. Fields with no commit intent yet keep it
// here and send it nowhere. `initial` is the engine's value, such as a default port.
export function useDraft(initial = "") {
	return useState(initial);
}

// Frame: a disclosure, such as a run of an agent expanded to its steps. The engine does not know
// which are open; each one starts collapsed and the user toggles it.
export type Disclosure = "collapsed" | "expanded";

export function useDisclosure() {
	const [disclosure, set] = useState<Disclosure>("collapsed");
	return {
		disclosure,
		toggle: () =>
			set((now) => (now === "collapsed" ? "expanded" : "collapsed")),
	};
}

// Frame: the measured viewport class. Below 820px the phone layout replaces the sidebar; below
// 1180px the desktop drops the reading pane and the thread's reply panel. The only breakpoints.
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

// Input: the web address bar. A path the browser loaded or went back to is input the engine has
// not confirmed yet, so the view answering an open_path intent replaces the history entry, while a
// view answering any other intent pushes one. Web only; Tauri has no address bar.
export type LastSent = "open-path" | "other";
export type AddressStep = "keep" | "push" | "replace";

export function addressStep(
	shown: string,
	path: string,
	lastSent: LastSent,
): AddressStep {
	if (shown === path) return "keep";
	return lastSent === "open-path" ? "replace" : "push";
}

// Startup sends open_path for the loaded address, so the first views answer it.
export function addressBar() {
	let lastSent: LastSent = "open-path";
	return {
		sent(kind: LastSent) {
			lastSent = kind;
		},
		step: (shown: string, path: string) => addressStep(shown, path, lastSent),
	};
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
