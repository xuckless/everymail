import { unset } from "../../engine";
import type { AgentResults, ResultItem } from "../../gen/ui/v1/agents_pb";
import { References } from "../../ui/Reference";
import { Row, Rows } from "../../ui/Rows";
import { Stats } from "../../ui/Stats";
import { BriefingBody, Byline, EditionTabs } from "../Briefing";

// What the agent produced: News Bearer's briefings as on the Briefing screen, or the other agents'
// groups of items, each with what it is about.
export function Results({ results }: { results: AgentResults }) {
	const { state } = results;
	switch (state.case) {
		case "briefing":
			return (
				<div className="pane-sections">
					<div className="results-briefing-head">
						<span className="sm muted grow">
							<Byline state={state.value.state} />
						</span>
						<EditionTabs view={state.value} />
					</div>
					<BriefingBody view={state.value} />
				</div>
			);
		case "groups":
			return (
				<div className="pane-sections">
					<Stats stats={state.value.stats} />
					{state.value.groups.map((group) => (
						<section key={group.title} className="pane-section">
							<h2 className="label">{group.title}</h2>
							<Rows label={group.title} className="results">
								{group.items.map((item, i) => (
									<Row key={item.id} index={i}>
										<Item item={item} />
									</Row>
								))}
							</Rows>
						</section>
					))}
				</div>
			);
		case "empty":
			return <p className="sm subtle settle pane-empty">{state.value.text}</p>;
		case undefined:
			return unset("AgentResults.state");
	}
}

function Item({ item }: { item: ResultItem }) {
	return (
		<article className="result">
			<span className="result-what">
				<span className="strong">{item.title}</span>
				<span className="sm muted">{item.detail}</span>
			</span>
			<References references={item.references} className="sm result-refs" />
			<span className="xs subtle result-time">{item.time}</span>
		</article>
	);
}
