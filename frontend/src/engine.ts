// The engine client: the Frontend's only connection, with one implementation per target behind
// one interface. Nothing above this module knows the target.
//   Tauri: raw-bytes invoke for intents, a Channel for views, bodies on the `body` URI scheme.
//   Web: the engine compiled to WASM in a Worker, bodies answered by a Service Worker that asks
//   that Worker for the bytes (body-sw.ts), and the address bar following the engine's path.

import { create, type MessageInitShape, toBinary } from "@bufbuild/protobuf";
import { Channel, invoke, isTauri } from "@tauri-apps/api/core";
import { createContext, useContext } from "react";
import { IntentSchema } from "./gen/ui/v1/shell_pb";
import { addressBar } from "./presentation";

export type IntentKind = NonNullable<
	MessageInitShape<typeof IntentSchema>["kind"]
>;

export type Engine = {
	send(intent: IntentKind): void;
	// Called with the path of each rendered view. Web: the address bar follows. Tauri: nothing.
	showPath(path: string): void;
	// The iframe sandbox attribute of a message body. Tauri: "allow-scripts". Web: none, because
	// Chromium does not route a sandboxed iframe's navigation through the Service Worker; the
	// Service Worker's response carries `Content-Security-Policy: sandbox allow-scripts` instead,
	// which gives the body the same opaque origin with scripts only.
	bodySandbox: "allow-scripts" | undefined;
};

const encode = (intent: IntentKind) =>
	toBinary(IntentSchema, create(IntentSchema, { kind: intent }));

function connectTauri(onView: (bytes: Uint8Array) => void): Engine {
	const views = new Channel<ArrayBuffer>();
	views.onmessage = (bytes) => onView(new Uint8Array(bytes));
	invoke("subscribe", { views });
	return {
		send: (intent) => {
			invoke("intent", encode(intent));
		},
		showPath: () => {},
		bodySandbox: "allow-scripts",
	};
}

const bodyWorker = import.meta.env.DEV ? "/src/body-sw.ts" : "/body-sw.js";

function connectWeb(onView: (bytes: Uint8Array) => void): Engine {
	const worker = new Worker(new URL("./engine.worker.ts", import.meta.url), {
		type: "module",
	});
	// The Service Worker asks for a body with its request (`m-slides-4?theme=dark`) and a port; the engine Worker answers
	// on that port with the bytes.
	navigator.serviceWorker.onmessage = (message) => {
		const [port] = message.ports;
		if (typeof message.data === "string" && port)
			worker.postMessage(message.data, [port]);
	};
	// A view may hold a body iframe, so none renders before the Service Worker can answer it.
	const bodies = navigator.serviceWorker
		.register(bodyWorker, { type: "module", scope: "/" })
		.then(() => navigator.serviceWorker.ready)
		.then(
			() => undefined,
			(error: unknown) => reportError(error),
		);
	worker.onmessage = (message: MessageEvent<Uint8Array>) => {
		bodies.then(() => onView(message.data));
	};

	const address = addressBar();
	const send = (intent: IntentKind) => {
		address.sent(intent.case === "openPath" ? "open-path" : "other");
		worker.postMessage(encode(intent));
	};
	const openShownPath = () =>
		send({ case: "openPath", value: { path: location.pathname } });
	addEventListener("popstate", openShownPath);
	openShownPath();

	return {
		send,
		showPath: (path) => {
			switch (address.step(location.pathname, path)) {
				case "keep":
					return;
				case "push":
					return history.pushState(null, "", path);
				case "replace":
					return history.replaceState(null, "", path);
			}
		},
		bodySandbox: undefined,
	};
}

export function connectEngine(onView: (bytes: Uint8Array) => void): Engine {
	return isTauri() ? connectTauri(onView) : connectWeb(onView);
}

export const EngineContext = createContext<Engine | undefined>(undefined);

export function useEngine(): Engine {
	const engine = useContext(EngineContext);
	if (!engine) throw new Error("useEngine outside EngineContext");
	return engine;
}

// Screens send an intent in one line: `const send = useSend(); send({ case: "back", value: {} })`.
export const useSend = () => useEngine().send;

// An unset oneof or an UNSPECIFIED enum is a decode error, never a state.
export function unset(what: string): never {
	throw new Error(`The engine sent ${what} unset`);
}
