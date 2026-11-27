import { SQL } from "bun";
import type { ScheduledTime } from "./gen/api/v1/time_pb";
import { Refused } from "./rpc";

/** The UTC instant a client's scheduled time names, by Postgres's time zone database, so the server holds the
 * only copy of the zone rules. Postgres moves a wall time skipped by a daylight saving change forward by the
 * gap and takes the later of one that happens twice. An instant, date, time or zone that does not exist is
 * refused as a bad request. */
export async function instant(sql: SQL, time: ScheduledTime): Promise<Date> {
	const { at } = time;
	if (!at.case) throw new Error("unset oneof in api.v1.ScheduledTime");
	const value =
		at.case === "unixSeconds"
			? sql`to_timestamp(${at.value.toString()}::float8)`
			: sql`make_timestamptz(${at.value.year}, ${at.value.month}, ${at.value.day},
				${at.value.hour}, ${at.value.minute}, 0, ${at.value.zone})`;
	try {
		const [row]: [{ at: Date }] = await sql`SELECT ${value} AS at`;
		return row.at;
	} catch (error) {
		// Class 22 is a data exception: out of range or not recognized
		if (error instanceof SQL.PostgresError && error.errno?.startsWith("22"))
			throw new Refused(400, "no such instant, date, time or zone");
		throw error;
	}
}
