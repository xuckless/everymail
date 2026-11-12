import { SQL } from "bun";
import { HealthService } from "./gen/api/v1/health_pb";
import { check } from "./health";
import { serve } from "./http";
import { rpc } from "./rpc";

const databaseUrl = process.env.DATABASE_URL;
if (!databaseUrl) throw new Error("DATABASE_URL is required");

const sql = new SQL(databaseUrl, {
	max: 3,
	connectionTimeout: 3,
});

const table = new Map([rpc(HealthService.method.check, () => check(sql))]);

const server = serve(table, Number(process.env.PORT ?? 8787));
console.log(`listening on ${server.url}`);
