import type { SQL } from "bun";
import {
	backoffSeconds,
	type Event,
	failure,
	freeSlots,
	type Job,
	type Lane,
	type Lanes,
	laneOf,
	type Outcome,
	transition,
} from "./job";
import { claim, expireLeases, secondsUntilNext, settle } from "./jobs";

/** What the rest of the Backend sees of the executor, so a separate process can replace it. */
export type Executor = {
	/** Call after a transaction that enqueued or readied a job commits. */
	wake: () => void;
	stop: () => void;
};

/** setTimeout's largest delay; a later due time is reached by waking again. */
const MAX_DELAY_MS = 2 ** 31 - 1;

const aborted = (signal: AbortSignal) =>
	new Promise<never>((_, reject) =>
		signal.addEventListener("abort", () => reject(signal.reason), {
			once: true,
		}),
	);

/** The event an outcome stands for, and what `settle` needs besides the target. */
function settlement(
	lane: Lane,
	job: Job,
	outcome: Outcome,
): [Event, Parameters<typeof settle>[4]] {
	if (typeof outcome === "string") return [outcome, {}];
	if ("repeatInSeconds" in outcome)
		return ["repeat", { dueInSeconds: outcome.repeatInSeconds }];
	const event = failure(lane.retry, job.attempt, "fail");
	return [
		event,
		{
			reason: outcome.fail,
			dueInSeconds:
				event === "fail" ? backoffSeconds(lane.retry, job.attempt) : 0,
		},
	];
}

/** Runs jobs inside this process. Holds only how many jobs each lane is running; all job state is in Postgres.
 * Never polls: a pass runs when woken, and a timer sleeps until the next due time or lease end. */
export function createExecutor(
	sql: SQL,
	lanes: Lanes,
	userCap: number,
): Executor {
	const running = new Map<string, number>();
	let timer: ReturnType<typeof setTimeout> | undefined;
	let pass: "idle" | "busy" | "busy, then again" | "stopped" = "idle";

	const count = (kind: string, by: number) =>
		running.set(kind, (running.get(kind) ?? 0) + by);

	async function run(lane: Lane, job: Job) {
		count(job.kind, 1);
		const signal = AbortSignal.timeout(lane.timeoutSeconds * 1000);
		let outcome: Outcome;
		try {
			outcome = await Promise.race([lane.run(job, signal), aborted(signal)]);
		} catch (error) {
			console.error(
				`job ${job.id} ${job.kind}: attempt ${job.attempt} failed: ${error instanceof Error ? error.message : "thrown"}`,
			);
			outcome = { fail: signal.aborted ? "timed_out" : "threw" };
		}
		count(job.kind, -1);
		const [event, options] = settlement(lane, job, outcome);
		const target = transition("running", event);
		if (!target) throw new Error(`running refused ${event}`);
		await settle(sql, lane, job, target, options);
	}

	/** Claims what each lane has room for and starts it, then sleeps until the next due time. */
	async function once() {
		await expireLeases(sql, lanes);
		const claimed = await claim(
			sql,
			lanes,
			new Map(freeSlots(lanes, running)),
			userCap,
		);
		for (const job of claimed)
			run(laneOf(lanes, job.kind), job)
				.catch((error) => console.error(`job ${job.id}: not recorded`, error))
				.finally(wake);
		const seconds = await secondsUntilNext(
			sql,
			freeSlots(lanes, running).map(([kind]) => kind),
			userCap,
		);
		clearTimeout(timer);
		if (seconds !== undefined)
			timer = setTimeout(
				wake,
				Math.min(MAX_DELAY_MS, Math.max(0, seconds * 1000)),
			);
	}

	function wake() {
		if (pass === "stopped") return;
		if (pass !== "idle") {
			pass = "busy, then again";
			return;
		}
		pass = "busy";
		once()
			.catch((error) => console.error("executor pass failed", error))
			.finally(() => {
				if (pass === "stopped") return;
				const again = pass === "busy, then again";
				pass = "idle";
				if (again) wake();
			});
	}

	wake();
	return {
		wake,
		stop: () => {
			pass = "stopped";
			clearTimeout(timer);
		},
	};
}
