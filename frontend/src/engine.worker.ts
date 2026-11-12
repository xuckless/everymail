import init, { WebEngine } from "@engine";

init().then(() => {
	const engine = new WebEngine(self.location.origin, (bytes: Uint8Array) =>
		self.postMessage(bytes, { transfer: [bytes.buffer] }),
	);
	self.onmessage = (message: MessageEvent<Uint8Array>) =>
		engine.intent(message.data);
});
