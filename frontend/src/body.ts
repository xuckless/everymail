// What the engine Worker answers a body request with: the body's bytes (undefined for an unknown
// message) and the content policy that sandboxes it, both from the engine.
export type BodyAnswer = {
	bytes: Uint8Array<ArrayBuffer> | undefined;
	policy: string;
};
