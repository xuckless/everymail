export type Status =
	| "scheduled"
	| "ready"
	| "running"
	| "waiting_for_approval"
	| "waiting_for_event"
	| "paused";

export type Event =
	| "claim"
	| "succeed"
	| "fail"
	| "expire"
	| "exhaust"
	| "await_approval"
	| "await_event"
	| "resolve"
	| "pause"
	| "resume"
	| "repeat"
	| "cancel";

export type Target =
	| { status: Status }
	| { finished: "succeeded" | "failed" | "cancelled" };

const to = (status: Status): Target => ({ status });
const end = (finished: "succeeded" | "failed" | "cancelled"): Target => ({
	finished,
});

const TRANSITIONS: Record<Event, Partial<Record<Status, Target>>> = {
	claim: {
		scheduled: to("running"),
		ready: to("running"),
	},
	succeed: {
		running: end("succeeded"),
	},
	fail: {
		running: to("scheduled"),
	},
	expire: {
		running: to("ready"),
	},
	exhaust: {
		running: end("failed"),
	},
	await_approval: {
		running: to("waiting_for_approval"),
	},
	await_event: {
		running: to("waiting_for_event"),
	},
	resolve: {
		waiting_for_approval: to("ready"),
		waiting_for_event: to("ready"),
	},
	pause: {
		scheduled: to("paused"),
		ready: to("paused"),
	},
	resume: {
		paused: to("ready"),
	},
	repeat: {
		running: to("scheduled"),
	},
	cancel: {
		scheduled: end("cancelled"),
		ready: end("cancelled"),
		waiting_for_approval: end("cancelled"),
		waiting_for_event: end("cancelled"),
		paused: end("cancelled"),
	},
};

export const transition = (status: Status, event: Event): Target | undefined =>
	TRANSITIONS[event][status];

export const sources = (event: Event) =>
	Object.keys(TRANSITIONS[event]) as Status[];

/** Why an attempt failed. The `failure_reason` enum: each value is added here and to the enum, by migration, together. */
export type FailureReason = "timed_out" | "threw";

export type Outcome =
	| Extract<Event, "succeed" | "await_approval" | "await_event">
	| { fail: FailureReason }
	/** Done for now: the same row runs again after this many seconds. */
	| { repeatInSeconds: number };

export type RetryPolicy = {
	maxAttempts: number;
	baseSeconds: number;
	capSeconds: number;
};

/** Delay before the next try after attempt number `attempt` (1 for the first) failed. */
export const backoffSeconds = (policy: RetryPolicy, attempt: number) =>
	Math.min(policy.capSeconds, policy.baseSeconds * 2 ** (attempt - 1));

export const failure = (
	policy: RetryPolicy,
	attempt: number,
	retry: "fail" | "expire",
): Event => (attempt >= policy.maxAttempts ? "exhaust" : retry);

export type Job = {
	id: string;
	userId: string;
	kind: string;
	attempt: number;
	payload: Uint8Array;
};

export type Lane = {
	concurrency: number;
	/** Which of a user's lanes takes that user's room under the per-user cap first: higher goes first. */
	priority: number;
	timeoutSeconds: number;
	retry: RetryPolicy;
	/** Agent run lanes: a finished job's outcome is written to the run log as the job is deleted. */
	log: boolean;
	run: (job: Job, signal: AbortSignal) => Promise<Outcome>;
};

export type Lanes = Readonly<Record<string, Lane>>;

/** The lane of a job kind; a kind with no lane is a bug, so it throws. */
export const laneOf = (lanes: Lanes, kind: string): Lane => {
	const lane = lanes[kind];
	if (!lane) throw new Error(`no lane for job kind ${kind}`);
	return lane;
};

export const freeSlots = (
	lanes: Lanes,
	running: ReadonlyMap<string, number>,
): [kind: string, free: number][] =>
	Object.entries(lanes)
		.map(([kind, lane]): [string, number] => [
			kind,
			lane.concurrency - (running.get(kind) ?? 0),
		])
		.filter(([, free]) => free > 0);

/** A due job a claim may take. */
export type Candidate = {
	id: string;
	userId: string;
	kind: string;
};

/** Round robin over users in the order their candidates arrive, which is their turn order. Each round every
 * user with room left takes one job: from its highest-priority lane with a free slot, oldest due first.
 * Candidates come grouped by user and oldest due first within each. Stops at the first round that takes nothing. */
export function pick(
	candidates: Candidate[],
	lanes: Lanes,
	laneFree: ReadonlyMap<string, number>,
	userFree: ReadonlyMap<string, number>,
): Candidate[] {
	const lanesLeft = new Map(laneFree);
	const usersLeft = new Map(userFree);
	const queues = Map.groupBy(candidates, (c) => c.userId);
	// Stable sort: equal priorities keep their due order
	for (const queue of queues.values())
		queue.sort(
			(a, b) => laneOf(lanes, b.kind).priority - laneOf(lanes, a.kind).priority,
		);
	const picked: Candidate[] = [];
	for (;;) {
		const before = picked.length;
		for (const [userId, queue] of queues) {
			if ((usersLeft.get(userId) ?? 0) <= 0) continue;
			const i = queue.findIndex((c) => (lanesLeft.get(c.kind) ?? 0) > 0);
			const [job] = i < 0 ? [] : queue.splice(i, 1);
			if (!job) continue;
			picked.push(job);
			usersLeft.set(userId, (usersLeft.get(userId) ?? 0) - 1);
			lanesLeft.set(job.kind, (lanesLeft.get(job.kind) ?? 0) - 1);
		}
		if (picked.length === before) return picked;
	}
}
