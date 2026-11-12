import { create, toBinary } from "@bufbuild/protobuf";
import { JoinRequestSchema } from "../../gen/api/v1/waitlist_pb";
import { type FormState, stateOfReply } from "../waitlist-state";
import { must, show } from "./show";

const form = must<HTMLFormElement>("#join");
const email = must<HTMLInputElement>("input", form);
const button = must<HTMLButtonElement>("button", form);

const render = (state: FormState) => {
	show(button, state);
	const confirmed = state === "joined" || state === "already";
	button.disabled = confirmed || state === "sending";
	button.classList.toggle("done", confirmed);
	button.classList.toggle("pulse", confirmed);
};

const join = async (): Promise<FormState> => {
	try {
		const response = await fetch("/api.v1.WaitlistService/Join", {
			method: "POST",
			headers: { "content-type": "application/proto" },
			body: toBinary(
				JoinRequestSchema,
				create(JoinRequestSchema, { email: email.value }),
			),
		});
		return response.ok
			? stateOfReply(new Uint8Array(await response.arrayBuffer()))
			: "failed";
	} catch {
		return "failed";
	}
};

form.addEventListener("submit", async (event) => {
	event.preventDefault();
	render("sending");
	render(await join());
});
