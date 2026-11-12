import { unset, useSend } from "../engine";
import {
	type InvitationAnswer,
	InvitationReply,
	type Invitation as InvitationView,
} from "../gen/ui/v1/common_pb";
import { Button } from "./Button";
import { cx } from "./cx";
import { Dot, type Tone } from "./Dot";
import { useDisclosure } from "./disclosure";
import { pulseOn, useOutcome } from "./Outcome";
import { Overlap } from "./Overlap";
import { Segmented } from "./Segmented";

// A calendar invitation and the user's answer, in the three forms drawn:
//   thread: in the message that carries it, the answer as a segmented control;
//   card: the calendar's sidebar, Accept, Maybe and Decline until answered, then Change;
//   sheet: the phone's sheet, three large buttons with the chosen one filled.
// Every answer is Intent.answer_invitation; the engine's next view carries it. What the card offers
// is the engine's (Invitation.prompt): the answers, or Change, which asks for them again.
type Answer = Exclude<InvitationAnswer["state"]["case"], undefined>;

// The three answers, with the weight each has on the card.
const answers: {
	answer: Answer;
	reply: InvitationReply;
	label: string;
	card: "primary" | "soft" | "ghost";
}[] = [
	{
		answer: "accepted",
		reply: InvitationReply.ACCEPT,
		label: "Accept",
		card: "primary",
	},
	{
		answer: "tentative",
		reply: InvitationReply.MAYBE,
		label: "Maybe",
		card: "soft",
	},
	{
		answer: "declined",
		reply: InvitationReply.DECLINE,
		label: "Decline",
		card: "ghost",
	},
];

export function Invitation({
	invitation,
	form,
}: {
	invitation: InvitationView;
	form: "thread" | "card" | "sheet";
}) {
	const answer = invitation.answer?.state.case ?? unset("InvitationAnswer");
	const send = useSend();
	const change = useDisclosure({ case: "eventId", value: invitation.eventId });
	const outcome = useOutcome({ case: "eventId", value: invitation.eventId });
	const reply = (each: InvitationReply) =>
		send({
			case: "answerInvitation",
			value: { eventId: invitation.eventId, reply: each },
		});
	const prompt = invitation.prompt.case ?? unset("Invitation.prompt");
	switch (form) {
		case "thread":
			return (
				<div className="tile invitation settle">
					<Status invitation={invitation} />
					<What invitation={invitation} />
					<Segmented
						kind="radio"
						label="Your answer"
						className="invitation-answer"
						segments={answers.map(({ answer: each, reply: r, label }) => ({
							key: each,
							label,
							selected: each === answer,
							onSelect: () => reply(r),
						}))}
					/>
				</div>
			);
		case "card":
			return (
				<div className="tile invitation invitation-card">
					<Status invitation={invitation} />
					<span className="strong">{invitation.title}</span>
					<span className="sm muted">{invitation.when}</span>
					<Overlap id={prompt}>
						{prompt === "ask" ? (
							<span className="invitation-buttons">
								{answers.map(({ answer: each, reply: r, label, card }) => (
									<Button
										key={each}
										variant={card}
										className={cx(card === "primary" && "grow")}
										onClick={() => reply(r)}
									>
										{label}
									</Button>
								))}
							</span>
						) : (
							<span className="invitation-buttons end">
								<Button variant="ghost" onClick={change.open}>
									Change
								</Button>
							</span>
						)}
					</Overlap>
				</div>
			);
		case "sheet":
			return (
				<div className="invitation-sheet">
					<span className="settle invitation-what">
						<span className="xs subtle">{invitation.invitedBy}</span>
						<span className="h2">{invitation.title}</span>
						<span className="sm muted">{invitation.when}</span>
						<span className="xs subtle">{invitation.details}</span>
					</span>
					<span className="settle invitation-large">
						{answers.map(({ answer: each, reply: r, label }) => (
							<button
								key={each}
								type="button"
								aria-pressed={each === answer}
								onClick={() => reply(r)}
								className={cx(
									"btn lg press",
									each === answer ? "primary" : "soft",
									each === answer && pulseOn(outcome),
								)}
							>
								{label}
							</button>
						))}
					</span>
				</div>
			);
	}
}

function Status({ invitation }: { invitation: InvitationView }) {
	return (
		<span className="xs subtle status-line">
			<Dot tone={tone(invitation.answer)} />
			{invitation.status}
		</span>
	);
}

function What({ invitation }: { invitation: InvitationView }) {
	return (
		<span className="invitation-what">
			<span className="invitation-title">{invitation.title}</span>
			<span className="sm muted">{invitation.when}</span>
			<span className="xs subtle">{invitation.details}</span>
		</span>
	);
}

function tone(answer: InvitationAnswer | undefined): Tone {
	switch (answer?.state.case) {
		case "unanswered":
			return "wait";
		case "accepted":
			return "done";
		case "tentative":
			return "wait";
		case "declined":
			return "idle";
		case undefined:
			return unset("InvitationAnswer");
	}
}
