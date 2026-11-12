import type { AgentDraft as AgentDraftView } from "../gen/ui/v1/common_pb";
import { ApprovalActions, approvalOf, approvalTone } from "./ApprovalActions";
import { Dot } from "./Dot";
import { Paragraphs } from "./Paragraphs";

// A reply an agent drafted, in the thread it answers or the reading pane: it is an approval item,
// with the approval actions of a reply to send.
export function AgentDraft({ draft }: { draft: AgentDraftView }) {
	return (
		<div className="tile agent-draft">
			<span className="xs subtle status-line">
				<Dot tone={approvalTone(approvalOf(draft.state))} />
				{draft.status}
			</span>
			<Paragraphs texts={draft.paragraphs} />
			<span className="actions">
				<ApprovalActions
					approvalId={draft.approvalId}
					state={draft.state}
					action="send"
				/>
			</span>
		</div>
	);
}
