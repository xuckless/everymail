import { useSend } from "../engine";
import type { ConnectionView, Navigation } from "../gen/ui/v1/shell_pb";
import { IconButton } from "../ui/Button";
import { Logo } from "../ui/Logo";
import { ConnectionStatus } from "./ConnectionStatus";
import { places } from "./places";

// The narrow desktop rail of the thread, agents, agent builder and settings screens.
export function Rail({
	nav,
	connection,
}: {
	nav: Navigation;
	connection: ConnectionView | undefined;
}) {
	const send = useSend();
	return (
		<nav className="rail" aria-label="Main">
			<Logo brand={{ kind: "mark" }} size={22} className="rail-mark" />
			{places.map((place) => (
				<IconButton
					key={place.destination}
					icon={place.icon}
					label={place.label}
					className="rail-item"
					aria-current={
						place.destination === nav.destination ? "page" : undefined
					}
					onClick={() => send(place.intent)}
				/>
			))}
			<ConnectionStatus
				connection={connection}
				form="dot"
				className="rail-status"
			/>
		</nav>
	);
}
