import type { SQL } from "bun";
import type { Lane } from "./job";
import { retention } from "./retention";

/** The `job_kind` enum. Each kind is added here and to the enum, by migration, together. */
export type JobKind = "retention";

/** The most jobs one user has running at once, across all lanes, so one backlog cannot starve the rest.
 * Above the 3 concurrent agent runs a user may have, so those alone never fill it. */
export const MAX_RUNNING_PER_USER = 4;

export const lanes = (sql: SQL): Record<JobKind, Lane> => ({
	retention: retention(sql),
});
