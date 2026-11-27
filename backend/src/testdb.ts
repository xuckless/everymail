import { afterAll, beforeAll, beforeEach } from "bun:test";
import { SQL, type TransactionSQL } from "bun";
import type { Lane, Status } from "./job";
import { enqueue } from "./jobs";
import { migrate } from "./migrate";

/** A throwaway schema, migrated before the file's tests and dropped after, so no test touches the real tables.
 * Its hooks run before any the caller registers later. Four connections: a conflict test holds one in the code
 * under test, one in another transaction, and writes with a third. */
export function useSchema(name: string): SQL {
	const url = process.env.DATABASE_URL;
	if (!url) throw new Error("DATABASE_URL is required for the database tests");
	const schema = `${name}_test_${process.pid}`;
	const admin = new SQL(url, { max: 1 });
	const sql = new SQL(url, { max: 4, connection: { search_path: schema } });
	beforeAll(async () => {
		await admin.unsafe(`CREATE SCHEMA ${schema}`);
		await migrate(sql);
	});
	afterAll(async () => {
		await sql.close();
		await admin.unsafe(`DROP SCHEMA ${schema} CASCADE`);
		await admin.close();
	});
	return sql;
}

/** `sql` that runs `between` once, just before the `at`-th query sent through it (from 1, counting the queries
 * of its transactions), so a test puts another writer at an exact point inside the code under test. */
export function interleave(
	sql: SQL,
	at: number,
	between: () => Promise<unknown>,
): SQL {
	let sent = 0;
	const wrap = <T extends SQL>(target: T): T =>
		new Proxy(target, {
			apply: (query, _, args) => {
				sent++;
				const send = () => Reflect.apply(query, target, args);
				return sent === at ? between().then(send) : send();
			},
			get: (obj, key) => {
				if (key === "begin")
					return (work: (tx: TransactionSQL) => Promise<unknown>) =>
						obj.begin((tx) => work(wrap(tx)));
				const value = Reflect.get(obj, key);
				return typeof value === "function" ? value.bind(obj) : value;
			},
		});
	return wrap(sql);
}

/** The lane the job tests start from; a test changes what it needs with a spread. */
export const testLane: Lane = {
	concurrency: 2,
	priority: 0,
	timeoutSeconds: 30,
	retry: { maxAttempts: 2, baseSeconds: 10, capSeconds: 60 },
	log: true,
	run: async () => "succeed",
};

/** A schema with the `test` and `urgent` job kinds and two users, a and b, emptied of jobs, run log and
 * turns before each test. */
export function useJobs(name: string) {
	const sql = useSchema(name);
	const users = { a: "", b: "" };
	beforeAll(async () => {
		// An added enum value is usable only once the transaction that added it commits
		await sql`ALTER TYPE job_kind ADD VALUE 'test'`;
		await sql`ALTER TYPE job_kind ADD VALUE 'urgent'`;
		[{ id: users.a }, { id: users.b }] = await sql`
			INSERT INTO users (provider, subject, email)
			VALUES ('google', 'a', 'a@a.a'), ('google', 'b', 'b@b.b') RETURNING id`;
	});
	beforeEach(async () => {
		await sql`DELETE FROM jobs`;
		await sql`DELETE FROM run_log`;
		await sql`UPDATE users SET claimed_at = NULL`;
	});
	/** Enqueues a job, then moves it to `status` when that is not scheduled; running comes only from a claim. */
	const add = async ({
		due = 0,
		owner = users.a,
		kind = "test",
		status = "scheduled",
		payload = new Uint8Array(),
	}: {
		due?: number;
		owner?: string;
		kind?: string;
		status?: Exclude<Status, "running">;
		payload?: Uint8Array;
	} = {}) => {
		const id = await enqueue(sql, {
			userId: owner,
			kind,
			payload,
			due: { inSeconds: due },
		});
		await sql`UPDATE jobs SET status = ${status}::job_status WHERE id = ${id}`;
		return id;
	};
	return { sql, users, add };
}
