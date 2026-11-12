import { expect, test } from "bun:test";
import { create, type MessageInitShape, toBinary } from "@bufbuild/protobuf";
import { JoinResponseSchema } from "../gen/api/v1/waitlist_pb";
import { stateOfReply } from "./waitlist-state";

const reply = (outcome: MessageInitShape<typeof JoinResponseSchema>) =>
	toBinary(JoinResponseSchema, create(JoinResponseSchema, outcome));

test("each outcome maps to its state", () => {
	expect(stateOfReply(reply({ outcome: { case: "joined", value: {} } }))).toBe(
		"joined",
	);
	expect(
		stateOfReply(reply({ outcome: { case: "alreadyJoined", value: {} } })),
	).toBe("already");
	expect(
		stateOfReply(reply({ outcome: { case: "invalidAddress", value: {} } })),
	).toBe("invalid");
});

test("an unset outcome or undecodable bytes is failed", () => {
	expect(stateOfReply(new Uint8Array())).toBe("failed");
	expect(stateOfReply(new Uint8Array([0xff, 0xff, 0xff]))).toBe("failed");
});
