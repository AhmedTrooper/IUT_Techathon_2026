import type { UsageResponse } from "../types";

interface PowerMeterProps {
	usage: UsageResponse | null;
}

export function PowerMeter({ usage }: PowerMeterProps) {
	const totalWatts = usage?.total_current_watts ?? 0;
	const todayKwh = usage?.today_kwh ?? 0;
	const roomBreakdown = usage?.room_breakdown ?? [];

	// Map room IDs to display names
	const roomNames: Record<string, string> = {
		drawing_room: "Drawing Room",
		work_room_1: "Work Room 1",
		work_room_2: "Work Room 2",
	};

	// Max potential power to scale progress bars (15 devices * average wattage)
	const maxPotentialWatts = 6 * 60 + 9 * 15; // 360W + 135W = 495W

	return (
		<div className="rounded-3xl border border-[var(--line)] bg-[var(--chip-bg)] p-6 transition-all hover:shadow-lg">
			<h3 className="text-lg font-bold text-[var(--sea-ink)] mb-6 flex items-center gap-2">
				<span>🔋</span> Power Consumption Meter
			</h3>

			<div className="grid grid-cols-1 md:grid-cols-2 gap-6 mb-6">
				{/* Main Watts Gauge Card */}
				<div className="flex flex-col items-center justify-center p-6 rounded-2xl border border-[var(--line)] bg-[var(--header-bg)] relative overflow-hidden">
					<div className="absolute top-0 left-0 right-0 h-1 bg-gradient-to-r from-teal-400 to-cyan-500" />
					<span className="text-sm font-semibold text-[var(--sea-ink-soft)] uppercase tracking-wider mb-2">
						Current Power Draw
					</span>
					<span className="text-5xl font-black text-[var(--sea-ink)] tracking-tight">
						{totalWatts}{" "}
						<span className="text-lg font-bold text-teal-500">W</span>
					</span>
					<div className="w-full bg-[var(--line)] h-1.5 rounded-full mt-4 overflow-hidden">
						<div
							className="bg-gradient-to-r from-teal-400 to-cyan-500 h-full rounded-full transition-all duration-500"
							style={{
								width: `${Math.min(100, (totalWatts / maxPotentialWatts) * 100)}%`,
							}}
						/>
					</div>
				</div>

				{/* Energy Usage Card */}
				<div className="flex flex-col items-center justify-center p-6 rounded-2xl border border-[var(--line)] bg-[var(--header-bg)] relative overflow-hidden">
					<div className="absolute top-0 left-0 right-0 h-1 bg-gradient-to-r from-emerald-400 to-teal-500" />
					<span className="text-sm font-semibold text-[var(--sea-ink-soft)] uppercase tracking-wider mb-2">
						Today's Est. Usage
					</span>
					<span className="text-5xl font-black text-[var(--sea-ink)] tracking-tight">
						{todayKwh.toFixed(4)}{" "}
						<span className="text-lg font-bold text-emerald-500">kWh</span>
					</span>
					<span className="text-xs text-[var(--sea-ink-soft)] mt-3">
						Accumulated usage since 00:00:00 UTC
					</span>
				</div>
			</div>

			{/* Room-by-room breakdown */}
			<div>
				<h4 className="text-sm font-bold text-[var(--sea-ink-soft)] uppercase tracking-wider mb-4">
					Room Breakdown
				</h4>
				<div className="space-y-4">
					{roomBreakdown.map((roomData) => {
						const displayName = roomNames[roomData.room] || roomData.room;
						const percentage = Math.min(
							100,
							(roomData.current_watts / 165) * 100,
						); // Max room power is 2*60 + 3*15 = 165W

						return (
							<div key={roomData.room} className="space-y-1">
								<div className="flex justify-between text-sm font-semibold text-[var(--sea-ink)]">
									<span>{displayName}</span>
									<span>{roomData.current_watts} W</span>
								</div>
								<div className="w-full bg-[var(--line)] h-2 rounded-full overflow-hidden">
									<div
										className="bg-teal-400 dark:bg-teal-500 h-full rounded-full transition-all duration-500"
										style={{ width: `${percentage}%` }}
									/>
								</div>
							</div>
						);
					})}
					{roomBreakdown.length === 0 && (
						<p className="text-sm text-[var(--sea-ink-soft)] italic">
							No active power draw to breakdown.
						</p>
					)}
				</div>
			</div>
		</div>
	);
}
