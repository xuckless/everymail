import { Frame } from "../chrome/Frame";
import { unset } from "../engine";
import type { Navigation, ShellView } from "../gen/ui/v1/shell_pb";
import { AgentBuilder } from "./AgentBuilder";
import { Agents } from "./Agents";
import { chatCoversPhone } from "./agents/Chat";
import { Briefing } from "./Briefing";
import { Calendar } from "./Calendar";
import { Connect } from "./Connect";
import { Inbox } from "./Inbox";
import { Quarantine } from "./Quarantine";
import { Settings } from "./Settings";
import { SignIn } from "./SignIn";
import { Thread } from "./Thread";

// A screen gets its own view (the ShellView.screen case) and, inside the chrome, the navigation.
// It renders only what goes inside the frame; the frame and its chrome are chosen here, as drawn:
// the sidebar on the inbox, calendar, Quarantine and briefing, the rail elsewhere; the phone's
// tab bar on the four tab destinations, nothing on the screens with a back affordance (an agent's
// full-screen chat is one).
export type ScreenProps<View> = { view: View; nav: Navigation };

export function Screen({ view }: { view: ShellView }) {
	const { screen, connection } = view;
	const nav = () => view.navigation ?? unset("ShellView.navigation");
	switch (screen.case) {
		case "signIn":
			return (
				<Frame chrome="none" connection={connection}>
					<SignIn view={screen.value} />
				</Frame>
			);
		case "connect":
			return (
				<Frame chrome="none" connection={connection}>
					<Connect view={screen.value} />
				</Frame>
			);
		case "inbox":
			return (
				<Frame
					chrome="sidebar"
					phone="tabs"
					nav={nav()}
					connection={connection}
				>
					<Inbox view={screen.value} nav={nav()} />
				</Frame>
			);
		case "thread":
			return (
				<Frame chrome="rail" phone="bare" nav={nav()} connection={connection}>
					<Thread view={screen.value} nav={nav()} />
				</Frame>
			);
		case "agents":
			return (
				<Frame
					chrome="rail"
					phone={chatCoversPhone(screen.value) ? "bare" : "tabs"}
					nav={nav()}
					connection={connection}
				>
					<Agents view={screen.value} nav={nav()} />
				</Frame>
			);
		case "agentBuilder":
			return (
				<Frame chrome="rail" phone="bare" nav={nav()} connection={connection}>
					<AgentBuilder view={screen.value} nav={nav()} />
				</Frame>
			);
		case "settings":
			return (
				<Frame chrome="rail" phone="tabs" nav={nav()} connection={connection}>
					<Settings view={screen.value} nav={nav()} />
				</Frame>
			);
		case "calendar":
			return (
				<Frame
					chrome="sidebar"
					phone="tabs"
					nav={nav()}
					connection={connection}
				>
					<Calendar view={screen.value} nav={nav()} />
				</Frame>
			);
		case "quarantine":
			return (
				<Frame
					chrome="sidebar"
					phone="bare"
					nav={nav()}
					connection={connection}
				>
					<Quarantine view={screen.value} nav={nav()} />
				</Frame>
			);
		case "briefing":
			return (
				<Frame
					chrome="sidebar"
					phone="tabs"
					nav={nav()}
					connection={connection}
				>
					<Briefing view={screen.value} nav={nav()} />
				</Frame>
			);
		case undefined:
			return unset("ShellView.screen");
	}
}
