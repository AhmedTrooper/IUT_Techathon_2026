import { Fan, Lightbulb, Users, Tv, Coffee } from "lucide-react";
import type { Device } from "../types";

interface OfficeLayoutProps {
	devices: Device[];
	onToggle: (id: string) => void;
}

export function OfficeLayout({ devices, onToggle }: OfficeLayoutProps) {
	// Helper to find device status
	const getDevice = (roomId: string, type: "fan" | "light", index: number) => {
		return devices.find(
			(d) => d.room === roomId && d.device_type === type && d.id.endsWith(index.toString())
		);
	};

	const DeviceIcon = ({ device }: { device?: Device }) => {
		if (!device) return null;
		const isOn = device.status;
		const isFan = device.device_type === "fan";

		return (
			<button
				onClick={() => onToggle(device.id)}
				className={`absolute transform -translate-x-1/2 -translate-y-1/2 p-2 rounded-full cursor-pointer transition-all duration-300 hover:scale-110 z-20 ${
					isOn
						? isFan
							? "bg-teal-500/20 text-teal-400 shadow-[0_0_20px_rgba(45,212,191,0.4)]"
							: "bg-amber-400/20 text-amber-300 shadow-[0_0_25px_rgba(251,191,36,0.6)]"
						: "bg-black/40 text-gray-500 hover:bg-white/10"
				}`}
				title={`${device.name} (${isOn ? "ON" : "OFF"})`}
			>
				{isFan ? (
					<Fan
						size={20}
						className={isOn ? "animate-spin" : ""}
						style={{ animationDuration: isOn ? "1s" : "0s" }}
					/>
				) : (
					<Lightbulb size={20} />
				)}
			</button>
		);
	};

	return (
		<div className="w-full bg-[var(--surface)] border border-[var(--line)] rounded-3xl p-6 lg:p-8 overflow-hidden shadow-2xl relative mb-8">
			<div className="flex items-center justify-between mb-6 z-10 relative">
				<h2 className="text-xl font-black text-[var(--sea-ink)] tracking-tight">
					Live Floor Plan
				</h2>
				<div className="flex gap-4 text-xs font-bold text-[var(--sea-ink-soft)] uppercase tracking-wider">
					<span className="flex items-center gap-1.5"><Lightbulb size={14} className="text-amber-400"/> Lights</span>
					<span className="flex items-center gap-1.5"><Fan size={14} className="text-teal-400"/> Fans</span>
				</div>
			</div>

			{/* Floor Plan Container */}
			<div className="relative w-full aspect-[16/9] max-w-5xl mx-auto bg-neutral-900/5 dark:bg-black/40 rounded-2xl border-2 border-neutral-200/50 dark:border-neutral-800/50 p-4">
				
				{/* Global Layout Grid */}
				<div className="w-full h-full grid grid-cols-5 grid-rows-2 gap-4 relative">
					
					{/* ENTRY ARROW */}
					<div className="absolute bottom-[-15px] left-10 flex flex-col items-center animate-bounce z-30">
						<div className="text-[10px] font-black tracking-widest text-teal-500 mb-1">ENTRY</div>
						<div className="w-1 h-8 bg-teal-500"></div>
						<div className="w-3 h-3 border-b-2 border-r-2 border-teal-500 transform rotate-45 -mt-2"></div>
					</div>

					{/* DRAWING ROOM (Col 1-2, Row 1-2) */}
					<div className="col-span-2 row-span-2 bg-gradient-to-br from-white to-neutral-100 dark:from-neutral-900 dark:to-black rounded-xl border border-[var(--line)] relative overflow-hidden shadow-inner flex flex-col">
						<div className="absolute inset-0 opacity-10 bg-[radial-gradient(circle_at_center,_var(--sea-ink)_1px,_transparent_1px)] bg-[size:20px_20px]"></div>
						<div className="p-4 z-10 text-sm font-bold text-[var(--sea-ink-soft)] uppercase tracking-widest opacity-60 flex justify-between">
							Drawing Room
							<Coffee size={16} />
						</div>
						
						{/* Furniture */}
						<div className="absolute bottom-12 left-8 w-24 h-12 bg-neutral-200 dark:bg-neutral-800 rounded-lg border border-neutral-300 dark:border-neutral-700 opacity-70"></div>
						<div className="absolute bottom-8 left-12 w-16 h-8 bg-neutral-300 dark:bg-neutral-700 rounded opacity-70"></div>
						<div className="absolute top-16 right-8 w-8 h-20 bg-neutral-200 dark:bg-neutral-800 rounded opacity-70 flex items-center justify-center"><Tv size={14} className="text-neutral-400"/></div>

						{/* Devices (2 Fans, 3 Lights) */}
						<div className="absolute top-1/4 left-1/3"><DeviceIcon device={getDevice("drawing_room", "light", 1)} /></div>
						<div className="absolute top-1/4 right-1/4"><DeviceIcon device={getDevice("drawing_room", "light", 2)} /></div>
						<div className="absolute bottom-1/3 left-1/2"><DeviceIcon device={getDevice("drawing_room", "light", 3)} /></div>
						
						<div className="absolute top-1/2 left-1/3"><DeviceIcon device={getDevice("drawing_room", "fan", 1)} /></div>
						<div className="absolute top-1/2 right-1/3"><DeviceIcon device={getDevice("drawing_room", "fan", 2)} /></div>
					</div>

					{/* WORK ROOM 1 (Col 3-5, Row 1) */}
					<div className="col-span-3 row-span-1 bg-gradient-to-br from-white to-neutral-100 dark:from-neutral-900 dark:to-black rounded-xl border border-[var(--line)] relative overflow-hidden shadow-inner">
						<div className="p-4 z-10 text-sm font-bold text-[var(--sea-ink-soft)] uppercase tracking-widest opacity-60 flex justify-between">
							Work Room 1
							<Users size={16} />
						</div>
						
						{/* Desks */}
						<div className="absolute top-1/2 -translate-y-1/2 left-12 w-20 h-10 bg-neutral-200 dark:bg-neutral-800 rounded border border-neutral-300 dark:border-neutral-700 opacity-60"></div>
						<div className="absolute top-1/2 -translate-y-1/2 left-40 w-20 h-10 bg-neutral-200 dark:bg-neutral-800 rounded border border-neutral-300 dark:border-neutral-700 opacity-60"></div>
						<div className="absolute top-1/2 -translate-y-1/2 right-20 w-20 h-10 bg-neutral-200 dark:bg-neutral-800 rounded border border-neutral-300 dark:border-neutral-700 opacity-60"></div>

						{/* Devices (2 Fans, 3 Lights) */}
						<div className="absolute top-1/3 left-1/4"><DeviceIcon device={getDevice("work_room_1", "light", 1)} /></div>
						<div className="absolute top-1/3 left-1/2"><DeviceIcon device={getDevice("work_room_1", "light", 2)} /></div>
						<div className="absolute top-1/3 right-1/4"><DeviceIcon device={getDevice("work_room_1", "light", 3)} /></div>

						<div className="absolute bottom-1/3 left-1/3"><DeviceIcon device={getDevice("work_room_1", "fan", 1)} /></div>
						<div className="absolute bottom-1/3 right-1/3"><DeviceIcon device={getDevice("work_room_1", "fan", 2)} /></div>
					</div>

					{/* WORK ROOM 2 (Col 3-5, Row 2) */}
					<div className="col-span-3 row-span-1 bg-gradient-to-br from-white to-neutral-100 dark:from-neutral-900 dark:to-black rounded-xl border border-[var(--line)] relative overflow-hidden shadow-inner">
						<div className="p-4 z-10 text-sm font-bold text-[var(--sea-ink-soft)] uppercase tracking-widest opacity-60 flex justify-between">
							Work Room 2
							<Users size={16} />
						</div>

						{/* Desks */}
						<div className="absolute top-1/2 -translate-y-1/2 left-16 w-24 h-12 bg-neutral-200 dark:bg-neutral-800 rounded-full border border-neutral-300 dark:border-neutral-700 opacity-60"></div>
						<div className="absolute top-1/2 -translate-y-1/2 right-16 w-32 h-16 bg-neutral-200 dark:bg-neutral-800 rounded border border-neutral-300 dark:border-neutral-700 opacity-60"></div>

						{/* Devices (2 Fans, 3 Lights) */}
						<div className="absolute top-1/3 left-1/4"><DeviceIcon device={getDevice("work_room_2", "light", 1)} /></div>
						<div className="absolute bottom-1/3 left-1/2"><DeviceIcon device={getDevice("work_room_2", "light", 2)} /></div>
						<div className="absolute top-1/3 right-1/4"><DeviceIcon device={getDevice("work_room_2", "light", 3)} /></div>

						<div className="absolute top-1/2 left-1/3"><DeviceIcon device={getDevice("work_room_2", "fan", 1)} /></div>
						<div className="absolute top-1/2 right-1/3"><DeviceIcon device={getDevice("work_room_2", "fan", 2)} /></div>
					</div>

					{/* DOORS */}
					<div className="absolute top-1/2 left-[40%] w-2 h-10 bg-[var(--line)] -translate-x-1/2 -translate-y-1/2 z-20 rounded-full"></div>
					<div className="absolute bottom-[25%] left-[40%] w-2 h-10 bg-[var(--line)] -translate-x-1/2 z-20 rounded-full"></div>
				</div>
			</div>
		</div>
	);
}
