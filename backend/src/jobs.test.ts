import { expect, test } from "bun:test";
import type { SQL } from "bun";
import type { Status } from "./job";
import {
	claim,
	commit,
	enqueue,
	expireLeases,
	secondsUntilNext,
	settle,
} from "./jobs";
import { interleave, testLane, useJobs } from "./testdb";

const { sql, users, add } = useJobs("jobs");

const lanes = {
	test: testLane,
	urgent: { ...testLane, priority: 1, timeoutSeconds: 600 },
};
const logged = { log: true };
const unlogged = { log: false };
/** Far above anything a test runs, so only the tests of the cap meet it. */
const NO_CAP = 100;
/** Where another writer lands, by query number: claim sends the lock, the candidate read, then the write;
 * an expiry reads the expired jobs, then settles each. */
const CLAIM_WRITE = 3;
const EXPIRY_SETTLE = 2;

/** Claims with `free` slots (a number is the test lane's), the test lane's lease at `leaseSeconds`. */
const take = (
	free: number | Record<string, number> = 10,
	{
		cap = NO_CAP,
		leaseSeconds = 30,
		through = sql,
	}: { cap?: number; leaseSeconds?: number; through?: SQL } = {},
) =>
	claim(
		through,
		{ ...lanes, test: { ...testLane, timeoutSeconds: leaseSeconds } },
		new Map(Object.entries(typeof free === "number" ? { test: free } : free)),
		cap,
	);

const ids = (jobs: { id: string }[]) => jobs.map((j) => j.id);

const statusOf = async (id: string) =>
	(await sql`SELECT status FROM jobs WHERE id = ${id}`)[0]?.status;

const logOf = (id: string) =>
	sql`SELECT kind, outcome, failure_reason, attempt, payload FROM run_log WHERE id = ${id}`;

const runningCount = async () =>
	(await sql`SELECT count(*)::int AS n FROM jobs WHERE status = 'running'`)[0]
		.n;

const claimedAt = async (user: string) =>
	(await sql`SELECT claimed_at FROM users WHERE id = ${user}`)[0].claimed_at;

test("claims due jobs only, oldest due first, then by id, up to the free slots", async () => {
	const late = await add({ due: -5 });
	const early = await add({ due: -60 });
	const tie = await add({ due: -5 });
	await add({ due: 3600 });
	expect(ids(await take(1))).toEqual([early]);
	expect(ids(await take(10)).sort()).toEqual([late, tie].sort());
	expect(await take(10)).toEqual([]);
});

test("a claim counts the attempt and carries the payload", async () => {
	const id = await add({ payload: new Uint8Array([1, 2, 3]) });
	const [job] = await take(1);
	expect(job).toMatchObject({ id, userId: users.a, kind: "test", attempt: 1 });
	expect([...(job?.payload ?? [])]).toEqual([1, 2, 3]);
	expect(await statusOf(id)).toBe("running");
});

/** Whether a due job in each live status is claimed. */
const CLAIMABLE: Record<Exclude<Status, "running">, boolean> = {
	scheduled: true,
	ready: true,
	paused: false,
	waiting_for_approval: false,
	waiting_for_event: false,
};

test("only scheduled and ready jobs are claimed, and a running one never again", async () => {
	for (const [status, claimable] of Object.entries(CLAIMABLE)) {
		await sql`DELETE FROM jobs`;
		const id = await add({
			due: -1,
			status: status as keyof typeof CLAIMABLE,
		});
		expect([status, ids(await take())]).toEqual([
			status,
			claimable ? [id] : [],
		]);
	}
	expect(await take()).toEqual([]);
});

test("nothing is claimed past what the lanes, the due time or the cap allow", async () => {
	const rows: {
		name: string;
		due: number;
		free: Record<string, number>;
		cap: number;
	}[] = [
		{ name: "a job not yet due", due: 60, free: { test: 1 }, cap: NO_CAP },
		{
			name: "a lane with no free slot",
			due: -1,
			free: { urgent: 1 },
			cap: NO_CAP,
		},
		{ name: "no lane free", due: -1, free: {}, cap: NO_CAP },
		{ name: "a cap of 0", due: -1, free: { test: 1 }, cap: 0 },
	];
	for (const row of rows) {
		await sql`DELETE FROM jobs`;
		await add({ due: row.due });
		expect([row.name, await take(row.free, { cap: row.cap })]).toEqual([
			row.name,
			[],
		]);
	}
});

test("one user's large backlog does not starve another: users take turns across passes", async () => {
	for (let i = 0; i < 10; i++) await add({ due: -60 });
	await add({ due: -5, owner: users.b });
	const owners: string[] = [];
	for (let pass = 0; pass < 4; pass++) {
		const [job] = await take(1);
		if (!job) throw new Error("not claimed");
		owners.push(job.userId);
		await settle(sql, logged, job, { finished: "succeeded" });
	}
	// Once b has nothing due, a's backlog has every pass
	expect(owners).toEqual([users.a, users.b, users.a, users.a]);
});

test("turns go to whoever was claimed for longest ago, never claimed first, ties by user id", async () => {
	const rows: {
		name: string;
		a: string | null;
		b: string | null;
		first: "a" | "b";
	}[] = [
		{ name: "neither claimed yet", a: null, b: null, first: "a" },
		{ name: "never claimed goes first", a: "10 seconds", b: null, first: "b" },
		{
			name: "claimed longer ago goes first",
			a: "10 seconds",
			b: "20 seconds",
			first: "b",
		},
		{
			name: "a tie goes by user id",
			a: "5 seconds",
			b: "5 seconds",
			first: "a",
		},
	];
	for (const row of rows) {
		await sql`DELETE FROM jobs`;
		// b's job is older, so due time alone would always pick b
		await add({ due: -1 });
		await add({ due: -60, owner: users.b });
		await sql`
			UPDATE users SET claimed_at = now() - CASE id
				WHEN ${users.a} THEN ${row.a}::interval ELSE ${row.b}::interval END`;
		const [job] = await take(1);
		expect([row.name, job?.userId]).toEqual([row.name, users[row.first]]);
	}
});

test("a claim stamps the turn of only the users it took a job from", async () => {
	await add({ due: -1 });
	await add({ due: -1, owner: users.b });
	await take(1);
	expect(await claimedAt(users.a)).not.toBeNull();
	expect(await claimedAt(users.b)).toBeNull();
});

test("the cap counts a user's jobs running in every lane, full ones too, and leaves other users alone", async () => {
	await add({ due: -1, kind: "urgent" });
	expect(await take({ urgent: 1 }, { cap: 2 })).toHaveLength(1);
	for (let i = 0; i < 3; i++) await add({ due: -1 });
	await add({ due: -1, owner: users.b });
	// The urgent lane has no free slot now, yet its running job still takes a's room
	const claimed = await take({ test: 10 }, { cap: 2 });
	expect(claimed.map((j) => j.userId).sort()).toEqual(
		[users.a, users.b].sort(),
	);
	expect(await take({ test: 10 }, { cap: 2 })).toEqual([]);
});

test("a user's room left under the cap is shared by all its lanes", async () => {
	await add({ kind: "urgent" });
	await take({ urgent: 1 }, { cap: 3 });
	for (const kind of ["test", "test", "urgent", "urgent"]) await add({ kind });
	expect(await take({ test: 10, urgent: 10 }, { cap: 3 })).toHaveLength(2);
});

test("a user's last room goes to the higher-priority lane, under that lane's lease", async () => {
	await add({ due: -60 });
	const urgent = await add({ due: -1, kind: "urgent" });
	expect(ids(await take({ test: 1, urgent: 1 }, { cap: 1 }))).toEqual([urgent]);
	expect(
		await sql`SELECT lease_until > now() + interval '590 seconds' AS long FROM jobs WHERE id = ${urgent}`,
	).toEqual([{ long: true }]);
});

test("a job another transaction holds is skipped, never waited on, and the rest are claimed", async () => {
	const held = await add({ due: -60 });
	const next = await add({ due: -1 });
	await sql.begin(async (tx) => {
		await tx`SELECT id FROM jobs WHERE id = ${held} FOR UPDATE`;
		expect(ids(await take(2))).toEqual([next]);
	});
	expect(await statusOf(held)).toBe("scheduled");
	expect(ids(await take(1))).toEqual([held]);
});

test("a job changed between the claim's read and its write is not claimed, nor its user stamped", async () => {
	const rows: { name: string; change: (id: string) => Promise<unknown> }[] = [
		{
			name: "paused",
			change: (id) => sql`UPDATE jobs SET status = 'paused' WHERE id = ${id}`,
		},
		{
			name: "moved later",
			change: (id) =>
				sql`UPDATE jobs SET due_at = now() + interval '1 hour' WHERE id = ${id}`,
		},
		{ name: "deleted", change: (id) => sql`DELETE FROM jobs WHERE id = ${id}` },
	];
	for (const row of rows) {
		await sql`DELETE FROM jobs`;
		const id = await add({ due: -1 });
		const through = interleave(sql, CLAIM_WRITE, () => row.change(id));
		expect([row.name, await take(1, { through })]).toEqual([row.name, []]);
		expect([row.name, await claimedAt(users.a)]).toEqual([row.name, null]);
	}
});

test("a user deleted between the claim's read and its write loses its jobs, and the others are claimed", async () => {
	const [{ id: gone }] = await sql`
		INSERT INTO users (provider, subject, email) VALUES ('google', 'gone', 'g@g.g') RETURNING id`;
	await add({ due: -60, owner: gone });
	const kept = await add({ due: -1 });
	const through = interleave(
		sql,
		CLAIM_WRITE,
		() => sql`DELETE FROM users WHERE id = ${gone}`,
	);
	expect(ids(await take(2, { through }))).toEqual([kept]);
});

test("a second claim started while the first is between its read and its write cannot push a user past the cap", async () => {
	await add({ due: -1 });
	await add({ due: -1, kind: "urgent" });
	let second: Promise<unknown> = Promise.resolve();
	const through = interleave(sql, CLAIM_WRITE, async () => {
		let done = false;
		second = take({ urgent: 1 }, { cap: 1 }).finally(() => {
			done = true;
		});
		// Wait until the second claim has finished or is queued behind the first's lock
		while (
			!done &&
			(
				await sql`SELECT 1 FROM pg_locks WHERE locktype = 'advisory' AND NOT granted`
			).length === 0
		)
			await Bun.sleep(5);
	});
	await take({ test: 1 }, { cap: 1, through });
	await second;
	expect(await runningCount()).toBe(1);
});

test("two claims at once take every job between them, none twice", async () => {
	for (let i = 0; i < 6; i++) await add({ due: -1 });
	const [first, second] = await Promise.all([take(4), take(4)]);
	const taken = [...ids(first), ...ids(second)];
	expect(taken).toHaveLength(6);
	expect(new Set(taken).size).toBe(6);
	expect(
		await sql`SELECT count(*)::int AS n FROM jobs WHERE status = 'running' AND attempt = 1`,
	).toEqual([{ n: 6 }]);
});

test("a finish that lands between an expiry's read and its settle wins, and the expiry changes nothing", async () => {
	const id = await add();
	const [job] = await take(1, { leaseSeconds: -1 });
	if (!job) throw new Error("not claimed");
	const through = interleave(sql, EXPIRY_SETTLE, () =>
		settle(sql, logged, job, { finished: "succeeded" }),
	);
	await expireLeases(through, lanes);
	expect(await statusOf(id)).toBeUndefined();
	expect(await logOf(id)).toEqual([
		expect.objectContaining({ outcome: "succeeded", attempt: 1 }),
	]);
});

test("an expired lease returns the job to ready, then fails it once attempts run out", async () => {
	const id = await add();
	await take(1, { leaseSeconds: -1 });
	await expireLeases(sql, lanes);
	expect(await statusOf(id)).toBe("ready");
	const [again] = await take(1, { leaseSeconds: -1 });
	expect(again?.attempt).toBe(2);
	await expireLeases(sql, lanes);
	expect(await statusOf(id)).toBeUndefined();
	expect((await logOf(id))[0]).toMatchObject({
		outcome: "failed",
		failure_reason: "timed_out",
		attempt: 2,
	});
});

test("a lease still running is left alone", async () => {
	const id = await add();
	await take(1);
	await expireLeases(sql, lanes);
	expect(await statusOf(id)).toBe("running");
});

test("a finish from an older attempt changes nothing", async () => {
	const id = await add();
	const [stale] = await take(1, { leaseSeconds: -1 });
	await expireLeases(sql, lanes);
	const [current] = await take(1);
	if (!stale || !current) throw new Error("not claimed");
	expect(await settle(sql, logged, stale, { finished: "succeeded" })).toBe(
		false,
	);
	expect(await statusOf(id)).toBe("running");
	expect(await logOf(id)).toEqual([]);
	expect(await settle(sql, logged, current, { finished: "succeeded" })).toBe(
		true,
	);
	expect(await statusOf(id)).toBeUndefined();
});

test("a finish deletes the job and logs its outcome and payload under the job's id", async () => {
	const id = await add({ payload: new Uint8Array([7]) });
	const [job] = await take(1);
	if (!job) throw new Error("not claimed");
	expect(await settle(sql, logged, job, { finished: "succeeded" })).toBe(true);
	expect(await statusOf(id)).toBeUndefined();
	const [row] = await logOf(id);
	expect(row).toMatchObject({
		kind: "test",
		outcome: "succeeded",
		failure_reason: null,
		attempt: 1,
	});
	expect([...row.payload]).toEqual([7]);
});

test("a failed run is logged with its reason", async () => {
	const id = await add();
	const [job] = await take(1);
	if (!job) throw new Error("not claimed");
	expect(
		await settle(sql, logged, job, { finished: "failed" }, { reason: "threw" }),
	).toBe(true);
	expect((await logOf(id))[0]).toMatchObject({
		outcome: "failed",
		failure_reason: "threw",
	});
});

test("a lane that does not log deletes its finished job and writes nothing", async () => {
	for (const finished of ["succeeded", "failed", "cancelled"] as const) {
		const id = await add();
		const [job] = await take(1);
		if (!job) throw new Error("not claimed");
		expect(
			await settle(sql, unlogged, job, { finished }, { reason: "threw" }),
		).toBe(true);
		expect(await statusOf(id)).toBeUndefined();
		expect(await logOf(id)).toEqual([]);
	}
});

test("a failure without a reason is refused and leaves the job running", async () => {
	const id = await add();
	const [job] = await take(1);
	if (!job) throw new Error("not claimed");
	await expect(
		settle(sql, unlogged, job, { finished: "failed" }),
	).rejects.toThrow();
	expect(await statusOf(id)).toBe("running");
});

test("a retried attempt is scheduled after its backoff", async () => {
	const id = await add();
	const [job] = await take(1);
	if (!job) throw new Error("not claimed");
	await settle(
		sql,
		logged,
		job,
		{ status: "scheduled" },
		{ dueInSeconds: 3600 },
	);
	expect(await statusOf(id)).toBe("scheduled");
	expect(await take(1)).toEqual([]);
});

test("a repeat keeps the row and its attempt count, scheduled after its interval", async () => {
	const id = await add();
	const [job] = await take(1);
	if (!job) throw new Error("not claimed");
	await settle(
		sql,
		logged,
		job,
		{ status: "scheduled" },
		{ dueInSeconds: 3600 },
	);
	expect(
		await sql`SELECT status, attempt, due_at > now() + interval '59 minutes' AS later
			FROM jobs WHERE id = ${id}`,
	).toEqual([{ status: "scheduled", attempt: 1, later: true }]);
	expect(await logOf(id)).toEqual([]);
});

test("the next wake is the nearest due time or lease end", async () => {
	expect(await secondsUntilNext(sql, ["test"], NO_CAP)).toBeUndefined();
	await add({ due: 3600 });
	const due = await secondsUntilNext(sql, ["test"], NO_CAP);
	expect(due).toBeGreaterThan(3590);
	expect(due).toBeLessThanOrEqual(3600);
	// A full lane's due jobs do not wake the executor
	expect(await secondsUntilNext(sql, [], NO_CAP)).toBeUndefined();
	await add();
	await take(1, { leaseSeconds: 600 });
	const lease = await secondsUntilNext(sql, [], NO_CAP);
	expect(lease).toBeGreaterThan(590);
	expect(lease).toBeLessThanOrEqual(600);
});

test("commit wakes the executor once the enqueue is visible, and a rollback wakes nothing", async () => {
	let wakes = 0;
	const wake = () => {
		wakes++;
	};
	const enqueueIn = (tx: SQL) =>
		enqueue(tx, {
			userId: users.a,
			kind: "test",
			payload: new Uint8Array(),
			due: { inSeconds: 0 },
		});
	const id = await commit(sql, wake, async (tx) => {
		const added = await enqueueIn(tx);
		expect(wakes).toBe(0);
		return added;
	});
	expect(wakes).toBe(1);
	expect(await statusOf(id)).toBe("scheduled");
	await expect(
		commit(sql, wake, async (tx) => {
			await enqueueIn(tx);
			throw new Error("rolled back");
		}),
	).rejects.toThrow("rolled back");
	expect(wakes).toBe(1);
	expect(await sql`SELECT id FROM jobs`).toHaveLength(1);
});

test("a due job whose user is at the cap does not wake the executor, but a lease end still does", async () => {
	// a's running job is in the urgent lane, whose lease is 600 seconds; a's due job is in the test lane
	await add({ kind: "urgent" });
	await take({ urgent: 1 });
	await add();
	const blocked = await secondsUntilNext(sql, ["test"], 1);
	expect(blocked).toBeGreaterThan(590);
	expect(blocked).toBeLessThanOrEqual(600);
	expect(await secondsUntilNext(sql, ["test"], 2)).toBeLessThanOrEqual(0);
	// Another user under the cap with a due job wakes it now
	await add({ owner: users.b });
	expect(await secondsUntilNext(sql, ["test"], 1)).toBeLessThanOrEqual(0);
});
