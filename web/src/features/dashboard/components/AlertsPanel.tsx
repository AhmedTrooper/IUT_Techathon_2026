import type { Alert } from "../hooks/useDashboardData";

interface AlertsPanelProps {
	alerts: Alert[];
}

export function AlertsPanel({ alerts }: AlertsPanelProps) {
	return (
		<div className="rounded-3xl border border-[var(--line)] bg-[var(--chip-bg)] p-6 transition-all hover:shadow-lg">
			<div className="mb-4 flex items-center justify-between">
				<h3 className="text-lg font-bold text-[var(--sea-ink)] flex items-center gap-2">
					<span>⚠️</span> Active Alerts
				</h3>
				<span
					className={`inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-semibold ${
						alerts.length > 0
							? "bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-400"
							: "bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400"
					}`}
				>
					{alerts.length} {alerts.length === 1 ? "Alert" : "Alerts"}
				</span>
			</div>

			{alerts.length === 0 ? (
				<div className="flex flex-col items-center justify-center py-6 text-center text-[var(--sea-ink-soft)]">
					<span className="text-3xl mb-2">✅</span>
					<p className="text-sm font-medium">
						All systems normal. No active anomalies detected.
					</p>
				</div>
			) : (
				<div className="space-y-3">
					{alerts.map((alert) => (
						<div
							key={alert.id}
							className={`flex items-start gap-3 rounded-2xl p-4 border transition-all ${
								alert.level === "CRITICAL"
									? "bg-red-50/50 border-red-200 dark:bg-red-950/20 dark:border-red-900/30"
									: "bg-amber-50/50 border-amber-200 dark:bg-amber-950/20 dark:border-amber-900/30"
							}`}
						>
							<span
								className={`text-lg ${alert.level === "CRITICAL" ? "text-red-500" : "text-amber-500"}`}
							>
								{alert.level === "CRITICAL" ? "🚨" : "⚡"}
							</span>
							<div className="flex-1">
								<p className="text-sm font-semibold text-[var(--sea-ink)] leading-snug">
									{alert.message}
								</p>
								<span className="text-[10px] font-medium text-[var(--sea-ink-soft)]">
									Triggered at {new Date(alert.timestamp).toLocaleTimeString()}
								</span>
							</div>
						</div>
					))}
				</div>
			)}
		</div>
	);
}
