import { useState } from "react";

const API_BASE = import.meta.env.VITE_API_URL || "http://localhost:8080/api";

export function DemoControls({ onUpdate }: { onUpdate: () => void }) {
	const [offset, setOffset] = useState(0);

	const setTimeOffset = async (hours: number) => {
		try {
			await fetch(`${API_BASE}/alerts/demo-time`, {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify({ offset_hours: hours }),
			});
			setOffset(hours);
			onUpdate();
		} catch (e) {
			console.error("Failed to set demo time", e);
		}
	};

	return (
		<div className="fixed bottom-4 left-4 z-50 bg-black/80 backdrop-blur-md border border-white/20 p-4 rounded-2xl shadow-2xl text-white text-xs w-[280px]">
			<h4 className="font-bold mb-2 flex items-center justify-between">
				<span>⏳ Time Travel (Demo)</span>
				<span className="text-teal-400 bg-teal-400/20 px-2 py-0.5 rounded-full">
					{offset > 0 ? `+${offset}h` : offset === 0 ? "Real" : `${offset}h`}
				</span>
			</h4>
			<p className="text-gray-400 mb-3 leading-tight">
				Shift time to test alerts and office hours.
			</p>

			<div className="grid grid-cols-2 gap-2">
				<button
					type="button"
					onClick={() => setTimeOffset(0)}
					className={`py-1.5 px-2 rounded-lg font-semibold transition col-span-2 ${offset === 0 ? "bg-white text-black" : "bg-white/10 hover:bg-white/20"}`}
				>
					Reset to Real Time
				</button>
				<button
					type="button"
					onClick={() => setTimeOffset(-2)}
					className={`py-1.5 px-2 rounded-lg font-semibold transition ${offset === -2 ? "bg-blue-500 text-white" : "bg-white/10 hover:bg-white/20"}`}
				>
					-2 Hours
				</button>
				<button
					type="button"
					onClick={() => setTimeOffset(2)}
					className={`py-1.5 px-2 rounded-lg font-semibold transition ${offset === 2 ? "bg-red-500 text-white" : "bg-white/10 hover:bg-white/20"}`}
				>
					+2 Hours
				</button>
				<button
					type="button"
					onClick={() => setTimeOffset(-5)}
					className={`py-1.5 px-2 rounded-lg font-semibold transition ${offset === -5 ? "bg-indigo-500 text-white" : "bg-white/10 hover:bg-white/20"}`}
				>
					-5 Hours
				</button>
				<button
					type="button"
					onClick={() => setTimeOffset(5)}
					className={`py-1.5 px-2 rounded-lg font-semibold transition ${offset === 5 ? "bg-amber-500 text-white" : "bg-white/10 hover:bg-white/20"}`}
				>
					+5 Hours
				</button>
				<button
					type="button"
					onClick={() => setTimeOffset(-12)}
					className={`py-1.5 px-2 rounded-lg font-semibold transition ${offset === -12 ? "bg-cyan-500 text-white" : "bg-white/10 hover:bg-white/20"}`}
				>
					-12 Hours
				</button>
				<button
					type="button"
					onClick={() => setTimeOffset(12)}
					className={`py-1.5 px-2 rounded-lg font-semibold transition ${offset === 12 ? "bg-purple-500 text-white" : "bg-white/10 hover:bg-white/20"}`}
				>
					+12 Hours
				</button>
			</div>
		</div>
	);
}
