import { lazy, type ReactNode, Suspense } from "react";
import { type Chrome, Frame } from "../chrome/Frame";
import { match, unset } from "../engine";
import type { Navigation, ShellView } from "../gen/ui/v1/shell_pb";

// A screen gets its own view (the ShellView.screen case) and, inside the chrome, the navigation.
// It renders only what goes inside the frame; the frame and its chrome are chosen here: the sidebar
// on every desktop screen; the phone's tab bar on the tab destinations, nothing on the screens with
// a back affordance (a thread, Quarantine, the builder, Approvals and an agent's page). Each screen
// is its own chunk, loaded the first time it shows.
export type ScreenProps<View> = { view: View; nav: Navigation };

const SignIn = lazy(() =>
	import("./SignIn").then((m) => ({ default: m.SignIn })),
);
const Connect = lazy(() =>
	import("./Connect").then((m) => ({ default: m.Connect })),
);
const Inbox = lazy(() => import("./Inbox").then((m) => ({ default: m.Inbox })));
const Thread = lazy(() =>
	import("./Thread").then((m) => ({ default: m.Thread })),
);
const Agents = lazy(() =>
	import("./Agents").then((m) => ({ default: m.Agents })),
);
const AgentBuilder = lazy(() =>
	import("./AgentBuilder").then((m) => ({ default: m.AgentBuilder })),
);
const Settings = lazy(() =>
	import("./Settings").then((m) => ({ default: m.Settings })),
);
const Calendar = lazy(() =>
	import("./Calendar").then((m) => ({ default: m.Calendar })),
);
const Quarantine = lazy(() =>
	import("./Quarantine").then((m) => ({ default: m.Quarantine })),
);
const Approvals = lazy(() =>
	import("./Approvals").then((m) => ({ default: m.Approvals })),
);
const Briefing = lazy(() =>
	import("./Briefing").then((m) => ({ default: m.Briefing })),
);

const tabs: Chrome = { chrome: "sidebar", phone: "tabs" };
const bare: Chrome = { chrome: "sidebar", phone: "bare" };

export function Screen({ view }: { view: ShellView }) {
	const { chrome, content } = screenOf(view);
	return (
		<Frame view={view} chrome={chrome}>
			<Suspense fallback={null}>{content}</Suspense>
		</Frame>
	);
}

function screenOf(view: ShellView): { chrome: Chrome; content: ReactNode } {
	const { screen } = view;
	const nav = () => view.navigation ?? unset("ShellView.navigation");
	switch (screen.case) {
		case "signIn":
			return {
				chrome: { chrome: "none" },
				content: <SignIn view={screen.value} />,
			};
		case "connect":
			return {
				chrome: { chrome: "none" },
				content: <Connect view={screen.value} />,
			};
		case "inbox":
			return {
				chrome: tabs,
				content: <Inbox view={screen.value} nav={nav()} />,
			};
		case "thread":
			return {
				chrome: bare,
				content: <Thread view={screen.value} nav={nav()} />,
			};
		case "agents":
			return {
				chrome: match(screen.value.page, "AgentsView.page", {
					list: () => tabs,
					agent: () => bare,
				}),
				content: <Agents view={screen.value} nav={nav()} />,
			};
		case "agentBuilder":
			return {
				chrome: bare,
				content: <AgentBuilder view={screen.value} nav={nav()} />,
			};
		case "settings":
			return {
				chrome: tabs,
				content: <Settings view={screen.value} nav={nav()} />,
			};
		case "calendar":
			return {
				chrome: tabs,
				content: <Calendar view={screen.value} nav={nav()} />,
			};
		case "quarantine":
			return {
				chrome: bare,
				content: <Quarantine view={screen.value} nav={nav()} />,
			};
		case "approvals":
			return {
				chrome: bare,
				content: <Approvals view={screen.value} nav={nav()} />,
			};
		case "briefing":
			return {
				chrome: tabs,
				content: <Briefing view={screen.value} nav={nav()} />,
			};
		case undefined:
			return unset("ShellView.screen");
	}
}
