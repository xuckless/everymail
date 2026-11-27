import { expect, test } from "bun:test";
import {
	backoffSeconds,
	type Event,
	failure,
	freeSlots,
	type Lane,
	pick,
	type Status,
	sources,
	type Target,
	transition,
} from "./job";

const STATUSES: Status[] = [
	"scheduled",
	"ready",
	"running",
	"waiting_for_approval",
	"waiting_for_event",
	"paused",
];
const EVENTS: Event[] = [
	"claim",
	"succeed",
	"fail",
	"expire",
	"exhaust",
	"await_approval",
	"await_event",
	"resolve",
	"pause",
	"resume",
	"repeat",
	"cancel",
];

/** "event status" for every allowed move; every other pair is refused. */
const ALLOWED: Record<string, Target> = {
	"claim scheduled": { status: "running" },
	"claim ready": { status: "running" },
	"succeed running": { finished: "succeeded" },
	"fail running": { status: "scheduled" },
	"expire running": { status: "ready" },
	"exhaust running": { finished: "failed" },
	"await_approval running": { status: "waiting_for_approval" },
	"await_event running": { status: "waiting_for_event" },
	"resolve waiting_for_approval": { status: "ready" },
	"resolve waiting_for_event": { status: "ready" },
	"pause scheduled": { status: "paused" },
	"pause ready": { status: "paused" },
	"resume paused": { status: "ready" },
	"repeat running": { status: "scheduled" },
	"cancel scheduled": { finished: "cancelled" },
	"cancel ready": { finished: "cancelled" },
	"cancel waiting_for_approval": { finished: "cancelled" },
	"cancel waiting_for_event": { finished: "cancelled" },
	"cancel paused": { finished: "cancelled" },
};

test("every status and event pair moves as allowed or is refused", () => {
	for (const event of EVENTS) {
		for (const status of STATUSES) {
			expect([event, status, transition(status, event)]).toEqual([
				event,
				status,
				ALLOWED[`${event} ${status}`],
			]);
		}
	}
});

test("sources lists exactly the statuses an event moves", () => {
	expect(sources("claim")).toEqual(["scheduled", "ready"]);
	expect(sources("cancel")).not.toContain("running");
	expect(sources("expire")).toEqual(["running"]);
});

const policy = { maxAttempts: 4, baseSeconds: 10, capSeconds: 60 };

test("backoff doubles from the base and stops at the cap", () => {
	expect([1, 2, 3, 4, 5].map((a) => backoffSeconds(policy, a))).toEqual([
		10, 20, 40, 60, 60,
	]);
});

test("a failure is retried until the last attempt, which exhausts the job", () => {
	expect(failure(policy, 3, "fail")).toBe("fail");
	expect(failure(policy, 3, "expire")).toBe("expire");
	expect(failure(policy, 4, "fail")).toBe("exhaust");
	expect(failure(policy, 4, "expire")).toBe("exhaust");
});

const lane = (concurrency: number, priority = 0): Lane => ({
	concurrency,
	priority,
	timeoutSeconds: 30,
	retry: policy,
	log: false,
	run: async () => "succeed",
});

test("a full lane leaves the other lanes' slots untouched", () => {
	const lanes = { agent: lane(2), send: lane(3) };
	expect(freeSlots(lanes, new Map([["agent", 2]]))).toEqual([["send", 3]]);
	expect(
		freeSlots(
			lanes,
			new Map([
				["agent", 1],
				["send", 1],
			]),
		),
	).toEqual([
		["agent", 1],
		["send", 2],
	]);
});

/** Candidates as "id user kind", in turn order and oldest due first within a user; send outranks agent and
 * fetch, which share a priority. */
const PICKS: {
	name: string;
	candidates: string[];
	laneFree: [string, number][];
	userFree: [string, number][];
	picked: string[];
}[] = [
	{
		name: "nothing due picks nothing",
		candidates: [],
		laneFree: [["agent", 3]],
		userFree: [],
		picked: [],
	},
	{
		name: "users take one job each per round, in turn order",
		candidates: ["a1 A agent", "a2 A agent", "a3 A agent", "b1 B agent"],
		laneFree: [["agent", 3]],
		userFree: [
			["A", 5],
			["B", 5],
		],
		picked: ["a1", "b1", "a2"],
	},
	{
		name: "a user stops at its room",
		candidates: ["a1 A agent", "a2 A agent", "a3 A agent"],
		laneFree: [["agent", 10]],
		userFree: [["A", 2]],
		picked: ["a1", "a2"],
	},
	{
		name: "a user without room takes nothing, and the next user still does",
		candidates: ["a1 A agent", "b1 B agent"],
		laneFree: [["agent", 10]],
		userFree: [
			["A", 0],
			["B", 1],
		],
		picked: ["b1"],
	},
	{
		name: "the higher-priority lane takes a user's last room though the other lane's job is older",
		candidates: ["a1 A agent", "s1 A send"],
		laneFree: [
			["agent", 1],
			["send", 1],
		],
		userFree: [["A", 1]],
		picked: ["s1"],
	},
	{
		name: "a full higher-priority lane does not block the user's other lane",
		candidates: ["s1 A send", "a1 A agent"],
		laneFree: [["agent", 1]],
		userFree: [["A", 2]],
		picked: ["a1"],
	},
	{
		name: "a lane's slots are shared by all users",
		candidates: ["a1 A agent", "b1 B agent", "s1 B send"],
		laneFree: [
			["agent", 1],
			["send", 1],
		],
		userFree: [
			["A", 2],
			["B", 2],
		],
		picked: ["a1", "s1"],
	},
	{
		name: "no lane free picks nothing",
		candidates: ["a1 A agent", "s1 A send"],
		laneFree: [],
		userFree: [["A", 5]],
		picked: [],
	},
	{
		name: "a user missing from the room has none",
		candidates: ["a1 A agent", "b1 B agent"],
		laneFree: [["agent", 5]],
		userFree: [["B", 5]],
		picked: ["b1"],
	},
	{
		name: "lanes of equal priority keep the user's due order",
		candidates: ["f1 A fetch", "a1 A agent", "f2 A fetch"],
		laneFree: [
			["agent", 5],
			["fetch", 5],
		],
		userFree: [["A", 2]],
		picked: ["f1", "a1"],
	},
	{
		name: "a user whose lanes fill drops out while the next user goes on",
		candidates: ["s1 A send", "s2 A send", "a1 B agent", "a2 B agent"],
		laneFree: [
			["agent", 2],
			["send", 1],
		],
		userFree: [
			["A", 5],
			["B", 5],
		],
		picked: ["s1", "a1", "a2"],
	},
];

test("pick goes round robin over users, under their room, higher-priority lanes first", () => {
	const lanes = { agent: lane(10), fetch: lane(10), send: lane(10, 1) };
	for (const row of PICKS) {
		const candidates = row.candidates.map((c) => {
			const [id = "", userId = "", kind = ""] = c.split(" ");
			return { id, userId, kind };
		});
		const picked = pick(
			candidates,
			lanes,
			new Map(row.laneFree),
			new Map(row.userFree),
		).map((c) => c.id);
		expect([row.name, picked]).toEqual([row.name, row.picked]);
	}
});

/** mulberry32: a seeded generator, so every random case below is the same on every run. */
const seeded = (seed: number) => () => {
	seed = (seed + 0x6d2b79f5) | 0;
	let t = Math.imul(seed ^ (seed >>> 15), 1 | seed);
	t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
	return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
};

test("pick keeps every rule on random cases", () => {
	for (let seed = 1; seed <= 500; seed++) {
		const random = seeded(seed);
		const upTo = (n: number) => Math.floor(random() * (n + 1));
		const kinds = Array.from({ length: 1 + upTo(2) }, (_, i) => `k${i}`);
		const lanes = Object.fromEntries(kinds.map((k) => [k, lane(1, upTo(2))]));
		const users = Array.from({ length: 1 + upTo(4) }, (_, i) => `u${i}`);
		// Grouped by user in turn order; within a user, the order made is the due order
		const candidates = users.flatMap((userId) =>
			Array.from({ length: upTo(6) }, (_, i) => ({
				id: `${userId}-${i}`,
				userId,
				kind: kinds[upTo(kinds.length - 1)] ?? "k0",
			})),
		);
		// A lane or user left out of a map has no room, like one at 0
		const laneFree = new Map(
			kinds.filter(() => random() < 0.9).map((k) => [k, upTo(5)]),
		);
		const userFree = new Map(
			users.filter(() => random() < 0.9).map((u) => [u, upTo(4)]),
		);
		const picked = pick(candidates, lanes, laneFree, userFree);

		const counted = (key: (c: (typeof candidates)[number]) => string) => {
			const counts = new Map<string, number>();
			for (const c of picked) counts.set(key(c), (counts.get(key(c)) ?? 0) + 1);
			return counts;
		};
		const byLane = counted((c) => c.kind);
		const byUser = counted((c) => c.userId);
		const laneFull = (kind: string) =>
			(byLane.get(kind) ?? 0) >= (laneFree.get(kind) ?? 0);
		const userFull = (user: string) =>
			(byUser.get(user) ?? 0) >= (userFree.get(user) ?? 0);
		const unpicked = candidates.filter((c) => !picked.includes(c));
		const priority = (c: { kind: string }) => lanes[c.kind]?.priority ?? 0;
		const broken: string[] = [];

		if (new Set(picked).size !== picked.length) broken.push("picked twice");
		if (picked.some((c) => !candidates.includes(c)))
			broken.push("picked a non-candidate");
		for (const [kind, n] of byLane)
			if (n > (laneFree.get(kind) ?? 0)) broken.push(`lane ${kind} overfilled`);
		for (const [user, n] of byUser)
			if (n > (userFree.get(user) ?? 0)) broken.push(`user ${user} over room`);
		for (const c of unpicked)
			if (!laneFull(c.kind) && !userFull(c.userId))
				broken.push(`${c.id} left though its lane and user had room`);
		for (const user of users)
			for (const kind of kinds) {
				const queue = candidates.filter(
					(c) => c.userId === user && c.kind === kind,
				);
				const taken = queue.filter((c) => picked.includes(c));
				if (taken.some((c, i) => c !== queue[i]))
					broken.push(`${user} ${kind} skipped an older job`);
			}
		for (const p of picked)
			for (const c of unpicked)
				if (
					c.userId === p.userId &&
					priority(c) > priority(p) &&
					!laneFull(c.kind)
				)
					broken.push(`${p.id} taken before higher-priority ${c.id}`);
		// Round robin: (the user's picks so far, the user's turn) only ever increases along the output
		const seen = new Map<string, number>();
		const steps = picked.map((c) => {
			const round = seen.get(c.userId) ?? 0;
			seen.set(c.userId, round + 1);
			return round * users.length + users.indexOf(c.userId);
		});
		if (steps.some((step, i) => i > 0 && step <= (steps[i - 1] ?? 0)))
			broken.push("not round robin");

		expect([seed, broken]).toEqual([seed, []]);
	}
});
