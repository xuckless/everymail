import "./briefing.css";
import type { ReactNode } from "react";
import { match, unset, useSend } from "../engine";
import type {
	BriefingEdition,
	BriefingView,
	CalendarSection,
	LineSection,
	NewsSection,
	WaitingSection,
} from "../gen/ui/v1/briefing_pb";
import { useLayout } from "../presentation";
import { Button } from "../ui/Button";
import { cx } from "../ui/cx";
import { Dot } from "../ui/Dot";
import { Icon } from "../ui/Icon";
import { Overlap } from "../ui/Overlap";
import { Page } from "../ui/Page";
import { Segmented } from "../ui/Segmented";
import { stagger } from "../ui/stagger";
import type { ScreenProps } from "./Screen";

// News Bearer's briefing: the editions as tabs, the picked one below in two columns where the page
// is wide. Waiting items open Approvals, Open Quarantine and Open calendar their screens. The same
// tabs and edition are News Bearer's Results pane on its agent page.
export function Briefing({ view }: ScreenProps<BriefingView>) {
	return (
		<Page
			label="Briefing"
			title="Briefing"
			subtitle={<Byline state={view.state} />}
			actions={<EditionTabs view={view} />}
			className="page-wrap"
		>
			<BriefingBody view={view} />
		</Page>
	);
}

export function EditionTabs({ view }: { view: BriefingView }) {
	const send = useSend();
	return (
		<Segmented
			kind="tabs"
			label="Briefing time"
			touch={useLayout() === "phone"}
			className="briefing-tabs"
			segments={view.editions.map((tab) => ({
				key: tab.id,
				label: tab.label,
				selected: tab.selected,
				onSelect: () =>
					send({ case: "selectEdition", value: { editionId: tab.id } }),
			}))}
		/>
	);
}

export function BriefingBody({ view }: { view: BriefingView }) {
	return (
		<Overlap
			id={match(view.state, "BriefingView.state", {
				edition: (edition) => edition.id,
				none: () => "none",
			})}
			className="briefing-stack"
		>
			<State state={view.state} />
		</Overlap>
	);
}

export function Byline({ state }: { state: BriefingView["state"] }) {
	switch (state.case) {
		case "edition":
			return state.value.byline;
		case "none":
			return null;
		case undefined:
			return unset("BriefingView.state");
	}
}

function State({ state }: { state: BriefingView["state"] }) {
	switch (state.case) {
		case "edition":
			return <Edition edition={state.value} />;
		case "none":
			return <p className="sm subtle">No briefing has been written yet</p>;
		case undefined:
			return unset("BriefingView.state");
	}
}

function Section({
	index,
	title,
	children,
}: {
	index: number;
	title: string;
	children: ReactNode;
}) {
	return (
		<div className="briefing-section settle" style={stagger(index)}>
			<h2 className="h2">{title}</h2>
			{children}
		</div>
	);
}

// A section with nothing in it shows the engine's text for that.
function Empty({ text }: { text: string }) {
	return <p className="sm subtle">{text}</p>;
}

function Lines({ section, bold }: { section?: LineSection; bold: boolean }) {
	const state = section?.state;
	switch (state?.case) {
		case "lines":
			return state.value.lines.map((line) => (
				<p key={`${line.source}${line.text}`} className="briefing-line">
					{bold ? (
						<>
							<span className="strong">{line.source}</span>{" "}
							<span className="muted">{line.text}</span>
						</>
					) : (
						<>
							<span className="sm subtle">{line.source}</span>
							<span>{line.text}</span>
						</>
					)}
				</p>
			));
		case "empty":
			return <Empty text={state.value.text} />;
		case undefined:
			return unset("LineSection.state");
	}
}

function Waiting({ section }: { section?: WaitingSection }) {
	const send = useSend();
	const layout = useLayout();
	const state = section?.state;
	switch (state?.case) {
		case "waiting":
			return (
				<div className="briefing-waiting">
					{state.value.items.map((item) => (
						<button
							key={item.approvalId}
							type="button"
							className={cx(
								"menu-row press briefing-wait",
								layout === "phone" && "touch",
							)}
							onClick={() => send({ case: "openApprovals", value: {} })}
						>
							<Dot tone="wait" />
							<span className="grow two-line">
								<span>{item.title}</span>
								<span className="xs subtle">{item.detail}</span>
							</span>
							<Icon name="chevron-right" size="sm" className="subtle" />
						</button>
					))}
				</div>
			);
		case "empty":
			return <Empty text={state.value.text} />;
		case undefined:
			return unset("WaitingSection.state");
	}
}

function Events({ section }: { section: CalendarSection }) {
	const { state } = section;
	switch (state.case) {
		case "events":
			return (
				<div className="briefing-events">
					{state.value.events.map((event) => (
						<div key={`${event.time}${event.title}`} className="briefing-event">
							<span className="sm subtle briefing-time">{event.time}</span>
							<span className="grow two-line">
								<span>{event.title}</span>
								<span className="xs subtle">{event.detail}</span>
							</span>
						</div>
					))}
				</div>
			);
		case "empty":
			return <Empty text={state.value.text} />;
		case undefined:
			return unset("CalendarSection.state");
	}
}

// The news the user follows, by topic. An item has no address to open, so where to read it (its
// link label, which names the source) is text.
function News({ section }: { section?: NewsSection }) {
	const state = section?.state;
	switch (state?.case) {
		case "topics":
			return (
				<div className="briefing-topics">
					{state.value.topics.map((topic) => (
						<div key={topic.name} className="briefing-topic">
							<h3 className="label">{topic.name}</h3>
							{topic.items.map((item) => (
								<div key={item.headline} className="briefing-news">
									<span className="briefing-headline">{item.headline}</span>
									<span className="xs subtle">{item.linkLabel}</span>
								</div>
							))}
						</div>
					))}
				</div>
			);
		case "none":
			return <Empty text={state.value.text} />;
		case undefined:
			return unset("NewsSection.state");
	}
}

// Two columns where there is room: what needs reading on the left, the day and the newsletters on
// the right.
function Edition({ edition }: { edition: BriefingEdition }) {
	const send = useSend();
	const layout = useLayout();
	const calendar = edition.calendar ?? unset("BriefingEdition.calendar");
	return (
		<article className="briefing-edition">
			<div className="briefing-column">
				<Section index={0} title="What your agents did">
					<Lines section={edition.activity} bold />
					{edition.caughtMail && (
						<Button
							variant="soft"
							size={layout === "phone" ? "lg" : undefined}
							className="briefing-open"
							onClick={() => send({ case: "openQuarantine", value: {} })}
						>
							Open Quarantine
						</Button>
					)}
				</Section>
				<Section index={1} title="Waiting for you">
					<Waiting section={edition.waiting} />
				</Section>
				<Section index={2} title="Updates">
					<Lines section={edition.updates} bold />
				</Section>
				<Section index={3} title="News">
					<News section={edition.news} />
				</Section>
			</div>
			<div className="briefing-column">
				<Section index={4} title={calendar.title}>
					<Events section={calendar} />
					<button
						type="button"
						className="link sm briefing-open-calendar"
						onClick={() => send({ case: "openCalendar", value: {} })}
					>
						Open calendar
					</button>
				</Section>
				<Section index={5} title="Newsletters, condensed">
					<div className="briefing-reads">
						<Lines section={edition.newsletters} bold={false} />
					</div>
				</Section>
			</div>
		</article>
	);
}
