import type { SQL, TransactionSQL } from "bun";
import {
	type Candidate,
	type FailureReason,
	failure,
	type Job,
	type Lane,
	type Lanes,
	laneOf,
	pick,
	type Status,
	sources,
	type Target,
	transition,
} from "./job";

const SCHEDULED: Status = "scheduled";
const RUNNING: Status = "running";

/** Arbitrary constant naming the lock claims take turns on; `migrate.ts` holds the other. */
const CLAIM_LOCK = 7_310_002;

const statuses = (sql: SQL, list: Status[]) => sql.array(list, "TEXT");

/** When a job comes due: after a delay from Postgres `now()`, or at an instant, such as one `instant` (`time.ts`)
 * computed from a client's scheduled time. */
export type Due = { inSeconds: number } | { at: Date };

/** Adds a job. Runs in the caller's transaction, which `commit` opens so the executor wakes. */
export async function enqueue(
	sql: SQL,
	job: {
		userId: string;
		kind: string;
		payload: Uint8Array;
		due: Due;
	},
): Promise<string> {
	const { due } = job;
	const [row] = await sql`
		INSERT INTO jobs (user_id, kind, status, due_at, payload)
		VALUES (${job.userId}, ${job.kind}::job_kind, ${SCHEDULED}::job_status,
			${"at" in due ? sql`${due.at}::timestamptz` : sql`now() + make_interval(secs => ${due.inSeconds})`},
			${job.payload})
		RETURNING id`;
	return row.id;
}

/** Runs `work` in one transaction and wakes the executor once it commits; a rollback wakes nothing.
 * A write that enqueues or readies a job goes through this. It opens the transaction itself, so it takes
 * the pool, never a transaction. */
export async function commit<T>(
	sql: SQL,
	wake: () => void,
	work: (tx: TransactionSQL) => Promise<T>,
): Promise<T> {
	const result = await sql.begin(work);
	wake();
	return result;
}

/** Moves to running, under each lane's lease, what the lanes have room for, fairly across users, and commits.
 * Users take turns, longest since their last claim first; each has at most `userCap` jobs running across all
 * lanes, and its higher-priority lanes fill that room first. Claims take turns on one lock, so two executors
 * cannot both see a user's room and overfill it; rows `settle` or an expiry holds are skipped, never waited on. */
export async function claim(
	sql: SQL,
	lanes: Lanes,
	freeLane: ReadonlyMap<string, number>,
	userCap: number,
): Promise<Job[]> {
	return sql.begin(async (tx) => {
		await tx`SELECT pg_advisory_xact_lock(${CLAIM_LOCK})`;
		// At most a user's room per lane, so a large backlog does not make the fetch large
		const due: (Candidate & { room: number })[] = await tx`
		WITH busy AS (
			SELECT user_id, count(*)::int AS running FROM jobs
			WHERE status = ${RUNNING}::job_status GROUP BY user_id
		), due AS (
			SELECT id, user_id, kind, due_at,
				row_number() OVER (PARTITION BY user_id, kind ORDER BY due_at, id) AS nth
			FROM jobs
			WHERE kind::text = ANY(${tx.array([...freeLane.keys()], "TEXT")}) AND due_at <= now()
				AND status = ANY(${statuses(tx, sources("claim"))}::job_status[])
		)
		SELECT d.id, d.user_id AS "userId", d.kind::text AS kind,
			${userCap} - coalesce(b.running, 0) AS room
		FROM due d JOIN users u ON u.id = d.user_id LEFT JOIN busy b USING (user_id)
		WHERE d.nth <= ${userCap} - coalesce(b.running, 0)
		ORDER BY u.claimed_at NULLS FIRST, u.id, d.due_at, d.id`;
		const picked = pick(
			due,
			lanes,
			freeLane,
			new Map(due.map((c) => [c.userId, c.room])),
		);
		if (picked.length === 0) return [];
		const rows: {
			id: string;
			user_id: string;
			kind: string;
			attempt: number;
			payload: Uint8Array;
		}[] = await tx`
		WITH chosen AS (
			SELECT j.id, p.lease FROM jobs j
			JOIN unnest(
				${tx.array(
					picked.map((c) => c.id),
					"TEXT",
				)}::bigint[],
				${tx.array(
					picked.map((c) => laneOf(lanes, c.kind).timeoutSeconds),
					"TEXT",
				)}::float8[]
			) AS p(id, lease) ON p.id = j.id
			WHERE j.due_at <= now()
				AND j.status = ANY(${statuses(tx, sources("claim"))}::job_status[])
			FOR UPDATE OF j SKIP LOCKED
		), taken AS (
			UPDATE jobs SET status = ${RUNNING}::job_status, attempt = attempt + 1,
				lease_until = now() + make_interval(secs => chosen.lease)
			FROM chosen WHERE jobs.id = chosen.id
			RETURNING jobs.id, jobs.user_id, jobs.kind::text AS kind, jobs.attempt, jobs.payload
		), turned AS (
			UPDATE users SET claimed_at = now() WHERE id IN (SELECT user_id FROM taken)
		)
		SELECT * FROM taken`;
		return rows.map((r) => ({
			id: r.id,
			userId: r.user_id,
			kind: r.kind,
			attempt: r.attempt,
			payload: r.payload,
		}));
	});
}

/** Records how a claimed attempt ended. Applies only while that attempt still holds the job,
 * so a finish arriving after its lease expired changes nothing. Returns whether it applied.
 * A finish deletes the job and, in the same statement, writes the run log row for a logged lane,
 * with the reason when it failed. */
export async function settle(
	sql: SQL,
	lane: Pick<Lane, "log">,
	job: { id: string; attempt: number },
	target: Target,
	{
		dueInSeconds = 0,
		reason,
	}: { dueInSeconds?: number; reason?: FailureReason } = {},
): Promise<boolean> {
	const failed = "finished" in target && target.finished === "failed";
	if (failed && !reason)
		throw new Error(`job ${job.id} failed without a reason`);
	const rows =
		"finished" in target
			? await sql`
				WITH done AS (
					DELETE FROM jobs
					WHERE id = ${job.id} AND attempt = ${job.attempt} AND status = ${RUNNING}::job_status
					RETURNING id, user_id, kind, attempt, payload
				), logged AS (
					INSERT INTO run_log (id, user_id, kind, outcome, failure_reason, attempt, payload)
					SELECT id, user_id, kind, ${target.finished}::run_outcome,
						${failed ? reason : null}::failure_reason, attempt, payload
					FROM done WHERE ${lane.log}
				)
				SELECT 1 FROM done`
			: await sql`
				UPDATE jobs SET status = ${target.status}::job_status, lease_until = NULL,
					due_at = now() + make_interval(secs => ${dueInSeconds})
				WHERE id = ${job.id} AND attempt = ${job.attempt} AND status = ${RUNNING}::job_status
				RETURNING 1`;
	return rows.length > 0;
}

/** Every running job whose lease ran out counts as a failed attempt: back to ready, or failed when none remain. */
export async function expireLeases(sql: SQL, lanes: Lanes): Promise<void> {
	const rows: { id: string; kind: string; attempt: number }[] = await sql`
		SELECT id, kind, attempt FROM jobs
		WHERE status = ${RUNNING}::job_status AND lease_until <= now()`;
	for (const row of rows) {
		const lane = laneOf(lanes, row.kind);
		const target = transition(
			RUNNING,
			failure(lane.retry, row.attempt, "expire"),
		);
		if (!target) throw new Error("running refused a lease expiry");
		await settle(sql, lane, row, target, { reason: "timed_out" });
	}
}

/** Seconds until the next job of these kinds comes due or the next lease runs out, by Postgres's clock.
 * A user at the cap is woken for by its finishing job, so its due jobs are left out.
 * Zero or less means now; undefined means nothing is pending. */
export async function secondsUntilNext(
	sql: SQL,
	kinds: string[],
	userCap: number,
): Promise<number | undefined> {
	const [row]: { seconds: number | null }[] = await sql`
		SELECT EXTRACT(EPOCH FROM least(
			(SELECT min(due_at) FROM jobs j
				WHERE kind::text = ANY(${sql.array(kinds, "TEXT")})
					AND status = ANY(${statuses(sql, sources("claim"))}::job_status[])
					AND (SELECT count(*) FROM jobs r
						WHERE r.user_id = j.user_id AND r.status = ${RUNNING}::job_status) < ${userCap}),
			(SELECT min(lease_until) FROM jobs WHERE status = ${RUNNING}::job_status)
		) - now())::float8 AS seconds`;
	return row?.seconds ?? undefined;
}
