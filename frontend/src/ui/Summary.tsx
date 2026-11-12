import { modelSettings } from "../chrome/places";
import { unset, useSend } from "../engine";
import type { Summary as SummaryView } from "../gen/ui/v1/common_pb";
import { Icon } from "./Icon";

// Scribe's summary in its well. `label` is the caption of a ready summary, such as
// "Summary by Scribe" or "Thread summary by Scribe"; off shows the way to add a model key.
export function Summary({
	summary,
	label,
}: {
	summary: SummaryView | undefined;
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
	summary: SummaryView | undefined;
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
		case undefined:
			return unset("Summary");
	}
}
