import { fromBinary } from "@bufbuild/protobuf";
import { JoinResponseSchema } from "../gen/api/v1/waitlist_pb";

export type FormState =
	| "idle"
	| "sending"
	| "joined"
	| "already"
	| "invalid"
	| "failed";

export const stateOfReply = (bytes: Uint8Array): FormState => {
	try {
		switch (fromBinary(JoinResponseSchema, bytes).outcome.case) {
			case "joined":
				return "joined";
			case "alreadyJoined":
				return "already";
			case "invalidAddress":
				return "invalid";
			case undefined:
				return "failed";
		}
	} catch {
		return "failed";
	}
};
