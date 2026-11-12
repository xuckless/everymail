import { modelSettings } from "../chrome/places";
import { unset, useSend } from "../engine";
import type { ScribeSummary } from "../gen/ui/v1/common_pb";
import { Icon } from "./Icon";

// Scribe's summary in its well. `label` is the caption of a ready summary, such as
// "Summary by Scribe" or "Thread summary by Scribe"; pending shimmers while Scribe writes it; off
// shows the way to add a model key; failed says why Scribe could not write it.
export function Summary({
	summary,
	label,
}: {
	summary: ScribeSummary | undefined;
	label: string;
}) {
	return (
		<div className="well summary">
			<Icon name="summary" className="accent" />
			<SummaryText summary={summary} label={label} />
		</div>
	);
}

function SummaryText({
	summary,
	label,
}: {
	summary: ScribeSummary | undefined;
	label: string;
}) {
	const send = useSend();
	switch (summary?.state.case) {
		case "ready":
			return (
				<span className="summary-text">
					<span className="xs subtle">{label}</span>
					<span>{summary.state.value.text}</span>
				</span>
			);
		case "off":
			return (
				<span className="summary-text">
					<span className="xs subtle">Scribe is off</span>
					<span>
						Summaries come back once you add a model key.{" "}
						<button
							type="button"
							className="link"
							onClick={() => send(modelSettings)}
						>
							Add a key
						</button>
					</span>
				</span>
			);
		case "pending":
			return (
				<span className="summary-text">
					<span className="xs subtle">{label}</span>
					<span className="shimmer">{summary.state.value.label}</span>
				</span>
			);
		case "failed":
			return (
				<span className="summary-text">
					<span className="xs subtle">{label}</span>
					<span className="bad">{summary.state.value.reason}</span>
				</span>
			);
		case undefined:
			return unset("ScribeSummary");
	}
}
