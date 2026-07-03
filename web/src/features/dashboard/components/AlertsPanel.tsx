import type { Device } from "../types";

interface AlertsPanelProps {
	devices: Device[];
}

export function AlertsPanel({ devices }: AlertsPanelProps) {
	// Compute active alerts
	const alerts: { type: "warning" | "error"; message: string; time: string }[] =
		[];

	// 1. Check office hours (9 AM - 5 PM)
	const now = new Date();
	const currentHour = now.getHours();
	const isAfterHours = currentHour < 9 || currentHour >= 17;

	if (isAfterHours) {
		const activeOutsideHours = devices.filter((d) => d.status);
		if (activeOutsideHours.length > 0) {
			alerts.push({
				type: "warning",
				message: `After-Hours Usage: ${activeOutsideHours.length} device(s) are active outside of office hours (9 AM - 5 PM).`,
				time: now.toLocaleTimeString(),
			});
		}
	}

	// 2. Check rooms with all devices ON for > 2 hours
	const rooms = [
		{ id: "drawing_room", name: "Drawing Room" },
		{ id: "work_room_1", name: "Work Room 1" },
		{ id: "work_room_2", name: "Work Room 2" },
	];

	rooms.forEach((room) => {
		const roomDevices = devices.filter((d) => d.room === room.id);
		const allOn = roomDevices.length > 0 && roomDevices.every((d) => d.status);

		if (allOn) {
			// Find the oldest last_changed timestamp in the room
			const oldestChange = roomDevices.reduce((oldest, current) => {
				const currentDate = new Date(current.last_changed);
				return currentDate < oldest ? currentDate : oldest;
			}, new Date());

			const diffMs = now.getTime() - oldestChange.getTime();
			const diffHours = diffMs / (1000 * 60 * 60);

			if (diffHours >= 2) {
				alerts.push({
					type: "error",
					message: `All devices in ${room.name} have been ON continuously for over 2 hours.`,
					time: oldestChange.toLocaleTimeString(),
				});
			}
		}
	});

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
							key={alert.message}
							className={`flex items-start gap-3 rounded-2xl p-4 border transition-all ${
								alert.type === "error"
									? "bg-red-50/50 border-red-200 dark:bg-red-950/20 dark:border-red-900/30"
									: "bg-amber-50/50 border-amber-200 dark:bg-amber-950/20 dark:border-amber-900/30"
							}`}
						>
							<span
								className={`text-lg ${alert.type === "error" ? "text-red-500" : "text-amber-500"}`}
							>
								{alert.type === "error" ? "🚨" : "⚡"}
							</span>
							<div className="flex-1">
								<p className="text-sm font-semibold text-[var(--sea-ink)] leading-snug">
									{alert.message}
								</p>
								<span className="text-[10px] font-medium text-[var(--sea-ink-soft)]">
									Triggered at {alert.time}
								</span>
							</div>
						</div>
					))}
				</div>
			)}
		</div>
	);
}
