import { Channel, invoke, isTauri } from "@tauri-apps/api/core";

export type Engine = { send(intent: Uint8Array): void };

function connectTauri(onView: (bytes: Uint8Array) => void): Engine {
	const views = new Channel<ArrayBuffer>();
	views.onmessage = (bytes) => onView(new Uint8Array(bytes));
	invoke("subscribe", { views });
	return {
		send: (intent) => {
			invoke("intent", intent);
		},
	};
}

function connectWeb(onView: (bytes: Uint8Array) => void): Engine {
	const worker = new Worker(new URL("./engine.worker.ts", import.meta.url), {
		type: "module",
	});
	worker.onmessage = (message: MessageEvent<Uint8Array>) =>
		onView(message.data);
	return { send: (intent) => worker.postMessage(intent) };
}

export function connectEngine(onView: (bytes: Uint8Array) => void): Engine {
	return isTauri() ? connectTauri(onView) : connectWeb(onView);
}
