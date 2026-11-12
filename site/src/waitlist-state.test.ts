import { expect, test } from "bun:test";
import { stateOfReply } from "./waitlist-state";

test("an unset outcome or undecodable bytes is failed", () => {
	expect(stateOfReply(new Uint8Array())).toBe("failed");
	expect(stateOfReply(new Uint8Array([0xff, 0xff, 0xff]))).toBe("failed");
});
