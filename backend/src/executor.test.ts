import { afterEach, expect, test } from "bun:test";
import { createExecutor, type Executor } from "./executor";
import type { Job, Outcome } from "./job";
import { testLane, useJobs } from "./testdb";

const { sql, users, add } = useJobs("executor");

/** Jobs the lanes have started, in order; each runs until the test finishes it. */
let started: { job: Job; finish: () => void }[] = [];
let arrived: (() => void)[] = [];
let executor: Executor | undefined;

const run = (job: Job) =>
	new Promise<Outcome>((resolve) => {
		started.push({ job, finish: () => resolve("succeed") });
		for (const notify of arrived.splice(0)) notify();
	});

/** Resolves once `count` jobs have started in all, never by timing. */
const startedUpTo = async (count: number) => {
	while (started.length < count)
		await new Promise<void>((notify) => arrived.push(notify));
	return started.slice(0, count);
};

const start = (concurrency: number, cap: number) => {
	executor = createExecutor(
		sql,
		{
			test: { ...testLane, concurrency, run },
			urgent: { ...testLane, concurrency, priority: 1, run },
		},
		cap,
	);
};

const owner = (n: number) => started[n]?.job.userId;

const running = async () =>
	(await sql`SELECT count(*)::int AS n FROM jobs WHERE status = 'running'`)[0]
		.n;

afterEach(async () => {
	executor?.stop();
	for (const { finish } of started) finish();
	// The finished attempts settle after the executor stops; wait for them before the next test empties the tables
	while ((await running()) > 0) await Bun.sleep(5);
	started = [];
	arrived = [];
});

test("with one slot and a cap of one, a backlog and a single job take turns until both drain", async () => {
	for (let i = 0; i < 3; i++) await add({ due: -60 });
	for (let i = 0; i < 2; i++) await add({ due: -1, owner: users.b });
	start(1, 1);
	for (let n = 1; n <= 5; n++) {
		const latest = (await startedUpTo(n))[n - 1];
		expect(await running()).toBe(1);
		latest?.finish();
	}
	expect([0, 1, 2, 3, 4].map(owner)).toEqual([
		users.a,
		users.b,
		users.a,
		users.b,
		users.a,
	]);
});

test("the cap holds while the lane has room, and a finish lets the user's next job start", async () => {
	for (let i = 0; i < 5; i++) await add({ due: -60 });
	await add({ due: -1, owner: users.b });
	start(4, 2);
	await startedUpTo(3);
	expect([0, 1, 2].map(owner)).toEqual([users.a, users.b, users.a]);
	expect(await running()).toBe(3);
	started[0]?.finish();
	await startedUpTo(4);
	expect(owner(3)).toBe(users.a);
});

test("a user's urgent job starts before its older one, which waits for the user's room", async () => {
	const older = await add({ due: -60 });
	const urgent = await add({ due: -1, kind: "urgent" });
	await add({ due: -1, owner: users.b });
	start(4, 1);
	await startedUpTo(2);
	expect(started.map((s) => [s.job.userId, s.job.kind])).toEqual([
		[users.a, "urgent"],
		[users.b, "test"],
	]);
	expect(started[0]?.job.id).toBe(urgent);
	started[0]?.finish();
	const [, , next] = await startedUpTo(3);
	expect(next?.job.id).toBe(older);
});
