import { expect, test } from "bun:test";
import { create } from "@bufbuild/protobuf";
import { ScheduledTimeSchema } from "./gen/api/v1/time_pb";
import { enqueue } from "./jobs";
import { Refused } from "./rpc";
import { useJobs } from "./testdb";
import { instant } from "./time";

const { sql, users } = useJobs("time");

const wall = ([year, month, day, hour, minute]: number[], zone: string) =>
	create(ScheduledTimeSchema, {
		at: {
			case: "wall",
			value: { year, month, day, hour, minute, zone },
		},
	});
const unix = (seconds: bigint) =>
	create(ScheduledTimeSchema, { at: { case: "unixSeconds", value: seconds } });

// New York springs forward at 02:00 on 8 March 2026 and falls back at 02:00 on 1 November 2026
const converted: [string, ReturnType<typeof wall>, string][] = [
	["a UTC instant", unix(1_780_000_000n), "2026-05-28T20:26:40.000Z"],
	[
		"standard time",
		wall([2026, 1, 15, 8, 0], "America/New_York"),
		"2026-01-15T13:00:00.000Z",
	],
	[
		"daylight time",
		wall([2026, 7, 15, 8, 0], "America/New_York"),
		"2026-07-15T12:00:00.000Z",
	],
	[
		"the minute before the gap",
		wall([2026, 3, 8, 1, 59], "America/New_York"),
		"2026-03-08T06:59:00.000Z",
	],
	[
		"inside the gap, moved forward by it",
		wall([2026, 3, 8, 2, 30], "America/New_York"),
		"2026-03-08T07:30:00.000Z",
	],
	[
		"the first minute after the gap",
		wall([2026, 3, 8, 3, 0], "America/New_York"),
		"2026-03-08T07:00:00.000Z",
	],
	[
		"inside the overlap, the later",
		wall([2026, 11, 1, 1, 30], "America/New_York"),
		"2026-11-01T06:30:00.000Z",
	],
	[
		"just after the overlap",
		wall([2026, 11, 1, 2, 0], "America/New_York"),
		"2026-11-01T07:00:00.000Z",
	],
	[
		"a southern zone's overlap",
		wall([2026, 4, 5, 2, 30], "Australia/Sydney"),
		"2026-04-04T16:30:00.000Z",
	],
	[
		"a half hour zone",
		wall([2026, 6, 1, 9, 0], "Asia/Kolkata"),
		"2026-06-01T03:30:00.000Z",
	],
];

test("a scheduled time becomes its UTC instant", async () => {
	for (const [, time, utc] of converted)
		expect((await instant(sql, time)).toISOString()).toBe(utc);
});

const refused: [string, ReturnType<typeof wall>][] = [
	["an unknown zone", wall([2026, 6, 1, 9, 0], "Nowhere/Town")],
	["a day the month lacks", wall([2026, 2, 30, 9, 0], "UTC")],
	["month 13", wall([2026, 13, 1, 9, 0], "UTC")],
	["minute 60", wall([2026, 6, 1, 9, 60], "UTC")],
	["an instant past the range", unix(9_000_000_000_000_000n)],
];

test("a time that does not exist is refused as a bad request", async () => {
	for (const [, time] of refused) {
		const error = await instant(sql, time).catch((e: unknown) => e);
		expect(error).toBeInstanceOf(Refused);
		expect((error as Refused).status).toBe(400);
	}
});

test("a job enqueued at an instant is stored at that instant", async () => {
	const at = await instant(sql, wall([2026, 3, 8, 2, 30], "America/New_York"));
	const id = await enqueue(sql, {
		userId: users.a,
		kind: "test",
		payload: new Uint8Array(),
		due: { at },
	});
	const [row] = await sql`SELECT due_at FROM jobs WHERE id = ${id}`;
	expect(row.due_at.toISOString()).toBe("2026-03-08T07:30:00.000Z");
});
