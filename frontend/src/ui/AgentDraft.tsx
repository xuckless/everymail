import type { AgentDraft as AgentDraftView } from "../gen/ui/v1/common_pb";
import { Button } from "./Button";
import { Dot } from "./Dot";
import { approvalDone, FillButton } from "./FillButton";
import { Paragraphs } from "./Paragraphs";

// A reply an agent drafted, in the thread it answers or the reading pane. Approve and send has no
// intent yet; when the engine reports it done, the button fills, its label crossfades to Sent and
// it pulses. `onEditWithAi` is where Edit with AI leads, if anywhere from this screen.
export function AgentDraft({
	draft,
	onEditWithAi,
}: {
	draft: AgentDraftView;
	onEditWithAi?: () => void;
}) {
	const done = approvalDone(draft.state);
	return (
		<div className="tile agent-draft">
			<span className="xs subtle status-line">
				<Dot tone={done ? "done" : "wait"} />
				{draft.status}
			</span>
			<Paragraphs texts={draft.paragraphs} />
			<span className="actions">
				<FillButton done={done} label="Approve and send" doneLabel="Sent" />
				<Button variant="soft" onClick={onEditWithAi}>
					Edit with AI
				</Button>
			</span>
		</div>
	);
}
