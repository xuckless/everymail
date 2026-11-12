import { unset } from "../../engine";
import type { AgentStatus } from "../../gen/ui/v1/common_pb";

// The active switch: on unless the user deactivated the agent.
export function isActive(status: AgentStatus | undefined): boolean {
	switch (status?.state.case) {
		case "running":
		case "idle":
		case "waiting":
			return true;
		case "deactivated":
			return false;
		case undefined:
			return unset("AgentStatus");
	}
}

// Run now waits while a run is under way.
export function isRunning(status: AgentStatus | undefined): boolean {
	switch (status?.state.case) {
		case "running":
			return true;
		case "idle":
		case "waiting":
		case "deactivated":
			return false;
		case undefined:
			return unset("AgentStatus");
	}
}
