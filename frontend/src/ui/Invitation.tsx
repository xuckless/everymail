import { unset } from "../engine";
import type {
	InvitationAnswer,
	Invitation as InvitationView,
} from "../gen/ui/v1/common_pb";
import { Button } from "./Button";
import { cx } from "./cx";
import { Dot, type Tone } from "./Dot";
import { Overlap } from "./Overlap";
import { Segmented } from "./Segmented";

// A calendar invitation and the user's answer, in the three forms drawn:
//   thread: in the message that carries it, the answer as a segmented control;
//   card: the calendar's sidebar, Accept, Maybe and Decline until answered, then Change;
//   sheet: the phone's sheet, three large buttons with the chosen one filled.
// Accept, Maybe, Decline and Change have no intent yet and are drawn without one.
type Answer = Exclude<InvitationAnswer["state"]["case"], undefined>;

const answers: { answer: Answer; label: string }[] = [
	{ answer: "accepted", label: "Accept" },
	{ answer: "tentative", label: "Maybe" },
	{ answer: "declined", label: "Decline" },
];

export function Invitation({
	invitation,
	form,
}: {
	invitation: InvitationView;
	form: "thread" | "card" | "sheet";
}) {
	const answer = invitation.answer?.state.case ?? unset("InvitationAnswer");
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
						segments={answers.map(({ answer: each, label }) => ({
							key: each,
							label,
							selected: each === answer,
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
					<Overlap id={answer === "unanswered" ? "asking" : "answered"}>
						{answer === "unanswered" ? (
							<span className="invitation-buttons">
								<Button variant="primary" className="grow">
									Accept
								</Button>
								<Button variant="soft">Maybe</Button>
								<Button variant="ghost">Decline</Button>
							</span>
						) : (
							<span className="invitation-buttons end">
								<Button variant="ghost">Change</Button>
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
						{answers.map(({ answer: each, label }) => (
							<button
								key={each}
								type="button"
								aria-pressed={each === answer}
								className={cx(
									"btn lg press",
									each === answer ? "primary pulse" : "soft",
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
