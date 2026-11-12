import "./briefing.css";
import type { ReactNode } from "react";
import { unset, useSend } from "../engine";
import type {
	BriefingEdition,
	BriefingView,
	CalendarSection,
	LineSection,
	WaitingSection,
} from "../gen/ui/v1/briefing_pb";
import { useLayout } from "../presentation";
import { Button } from "../ui/Button";
import { cx } from "../ui/cx";
import { Dot } from "../ui/Dot";
import { Icon } from "../ui/Icon";
import { Overlap } from "../ui/Overlap";
import { Segmented } from "../ui/Segmented";
import { stagger } from "../ui/stagger";
import type { ScreenProps } from "./Screen";

// News Bearer's briefing: the editions across the top, the picked one below. Waiting items open
// the agents screen, Open Quarantine and Open calendar their screens; the action buttons of the
// design's other states have no intent and are not drawn here.
export function Briefing({ view }: ScreenProps<BriefingView>) {
	const send = useSend();
	const layout = useLayout();
	return (
		<section className={cx("briefing", layout !== "phone" && "panel")}>
			<header className="briefing-head">
				<span className="grow briefing-title">
					<h1 className="h1">Briefing</h1>
					<Byline state={view.state} />
				</span>
				<Segmented
					kind="tabs"
					label="Briefing time"
					touch={layout === "phone"}
					className="briefing-tabs"
					segments={view.editions.map((tab) => ({
						key: tab.id,
						label: tab.label,
						selected: tab.selected,
						onSelect: () =>
							send({ case: "selectEdition", value: { editionId: tab.id } }),
					}))}
				/>
			</header>
			<div className="scroll noscroll briefing-scroll">
				<Overlap
					id={view.state.case === "edition" ? view.state.value.id : "none"}
					className="briefing-stack"
				>
					<State state={view.state} />
				</Overlap>
			</div>
		</section>
	);
}

function Byline({ state }: { state: BriefingView["state"] }) {
	switch (state.case) {
		case "edition":
			return <span className="sm muted">{state.value.byline}</span>;
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
		<div className="briefing-section settle stagger" style={stagger(index)}>
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
							key={`${item.title}${item.detail}`}
							type="button"
							className={cx(
								"menu-row press briefing-wait",
								layout === "phone" && "touch",
							)}
							onClick={() => send({ case: "openAgents", value: {} })}
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

function Edition({ edition }: { edition: BriefingEdition }) {
	const send = useSend();
	const calendar = edition.calendar ?? unset("BriefingEdition.calendar");
	return (
		<article className="briefing-edition">
			<Section index={0} title="What your agents did">
				<Lines section={edition.activity} bold />
				{edition.caughtMail && (
					<Button
						variant="soft"
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
			<Section index={2} title="Newsletters, condensed">
				<div className="briefing-reads">
					<Lines section={edition.newsletters} bold={false} />
				</div>
			</Section>
			<Section index={3} title="Updates">
				<Lines section={edition.updates} bold />
			</Section>
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
		</article>
	);
}
