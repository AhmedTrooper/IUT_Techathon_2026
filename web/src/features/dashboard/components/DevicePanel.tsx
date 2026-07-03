import { Fan, Lightbulb } from "lucide-react";
import type { Device } from "../types";

interface DevicePanelProps {
	devices: Device[];
	onToggle: (id: string) => void;
}

export function DevicePanel({ devices, onToggle }: DevicePanelProps) {
	const rooms = [
		{ id: "drawing_room", name: "Drawing Room" },
		{ id: "work_room_1", name: "Work Room 1" },
		{ id: "work_room_2", name: "Work Room 2" },
	];

	return (
		<div className="space-y-8">
			{rooms.map((room) => {
				const roomDevices = devices.filter((d) => d.room === room.id);

				return (
					<div
						key={room.id}
						className="rounded-3xl border border-[var(--line)] bg-[var(--chip-bg)] p-6 transition-all hover:shadow-lg"
					>
						<h3 className="text-xl font-black text-[var(--sea-ink)] mb-4 flex items-center gap-2 border-b border-[var(--line)] pb-3">
							<span className="h-2 w-2 rounded-full bg-teal-400" />
							{room.name}
						</h3>

						<div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-4">
							{roomDevices.map((device) => {
								const isOn = device.status;
								const isFan = device.device_type === "fan";

								return (
									<div
										key={device.id}
										className={`flex flex-col justify-between p-4 rounded-2xl border transition-all duration-300 ${
											isOn
												? "bg-[var(--header-bg)] border-teal-400/50 shadow-[0_0_15px_rgba(45,212,191,0.08)]"
												: "bg-transparent border-[var(--line)]"
										}`}
									>
										<div className="flex items-start justify-between mb-4">
											{/* Device Icon with Animation & Glow */}
											<div
												className={`p-3 rounded-xl transition-all duration-300 ${
													isOn
														? isFan
															? "bg-teal-50 dark:bg-teal-950/30 text-teal-500"
															: "bg-amber-50 dark:bg-amber-950/30 text-amber-500 shadow-[0_0_12px_rgba(245,158,11,0.2)]"
														: "bg-[var(--chip-bg)] text-[var(--sea-ink-soft)]"
												}`}
											>
												{isFan ? (
													<Fan
														className={`h-6 w-6 ${isOn ? "animate-spin" : ""}`}
														style={{ animationDuration: isOn ? "1.5s" : "0s" }}
													/>
												) : (
													<Lightbulb className="h-6 w-6" />
												)}
											</div>

											{/* Power Badge */}
											<span className="text-[10px] font-bold text-[var(--sea-ink-soft)] uppercase tracking-wider bg-[var(--header-bg)] border border-[var(--line)] px-2 py-0.5 rounded-full">
												{device.power_consumption}W
											</span>
										</div>

										<div>
											<h4 className="text-sm font-extrabold text-[var(--sea-ink)] mb-1">
												{device.name}
											</h4>
											<p className="text-[11px] text-[var(--sea-ink-soft)] mb-4">
												Last toggled:{" "}
												{new Date(device.last_changed).toLocaleTimeString()}
											</p>

											{/* Slider/Switch Toggle */}
											<button
												type="button"
												onClick={() => onToggle(device.id)}
												className={`w-full py-2.5 px-4 rounded-xl text-xs font-bold transition-all active:scale-[0.98] cursor-pointer ${
													isOn
														? "bg-teal-500 hover:bg-teal-600 text-white shadow-md shadow-teal-500/10"
														: "bg-[var(--header-bg)] hover:bg-[var(--line)] text-[var(--sea-ink)] border border-[var(--line)]"
												}`}
											>
												{isOn ? "Turn OFF" : "Turn ON"}
											</button>
										</div>
									</div>
								);
							})}
						</div>
					</div>
				);
			})}
		</div>
	);
}
