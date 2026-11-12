import type { MessageInitShape } from "@bufbuild/protobuf";
import { match, useSend } from "../engine";
import type {
	Disclosure,
	DisclosureTargetSchema,
} from "../gen/ui/v1/common_pb";

// A part of a screen the engine opens and closes (ui.v1.Disclosure): the two intents that change
// it and, given the view's Disclosure, whether it is open and the toggle its trigger sends.
export type DisclosureTargetInit = NonNullable<
	MessageInitShape<typeof DisclosureTargetSchema>["target"]
>;

export function useDisclosure(target: DisclosureTargetInit) {
	const send = useSend();
	const open = () =>
		send({ case: "openDisclosure", value: { target: { target } } });
	const close = () =>
		send({ case: "closeDisclosure", value: { target: { target } } });
	return {
		open,
		close,
		// The view's state of it, and the trigger's intent for that state.
		of: (disclosure: Disclosure | undefined, what: string) =>
			match(disclosure?.state ?? { case: undefined }, what, {
				closed: () => ({ isOpen: false, toggle: open }),
				open: () => ({ isOpen: true, toggle: close }),
			}),
	};
}
