import type { Stat } from "../gen/ui/v1/common_pb";

// Figures with their captions, in a row of tiles.
export function Stats({ stats }: { stats: Stat[] }) {
	return (
		<div className="stats">
			{stats.map((stat) => (
				<div key={stat.label} className="tile stat">
					<span className="stat-value">{stat.value}</span>
					<span className="xs subtle">{stat.label}</span>
				</div>
			))}
		</div>
	);
}
