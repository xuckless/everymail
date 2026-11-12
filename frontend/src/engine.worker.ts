// The web engine: the WASM build of the Rust engine in a Worker. It receives intents as bytes and
// body requests (`m-slides-4?theme=dark`) with a reply port, and posts views as bytes.

import init, { WebEngine } from "@engine";
import type { BodyAnswer } from "./body";

const engine = init().then(
	() =>
		new WebEngine(self.location.origin, (bytes: Uint8Array) =>
			self.postMessage(bytes, { transfer: [bytes.buffer] }),
		),
);

// Messages that arrive before the engine is ready wait for it, in order.
self.onmessage = async (message: MessageEvent<Uint8Array | string>) => {
	const ready = await engine;
	const { data } = message;
	if (typeof data !== "string") return ready.intent(data);
	const [port] = message.ports;
	if (!port) throw new Error("A body request without a reply port");
	const bytes = ready.body(data) as BodyAnswer["bytes"];
	const answer: BodyAnswer = { bytes, policy: WebEngine.bodySandbox() };
	port.postMessage(answer, bytes ? [bytes.buffer] : []);
};
