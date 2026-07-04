import { createFileRoute } from "@tanstack/react-router";
import { AlertsPanel } from "../features/dashboard/components/AlertsPanel";
import { DevicePanel } from "../features/dashboard/components/DevicePanel";
import { OfficeLayout } from "../features/dashboard/components/OfficeLayout";
import { PowerMeter } from "../features/dashboard/components/PowerMeter";
import { DemoControls } from "../features/dashboard/components/DemoControls";
import { useDashboardData } from "../features/dashboard/hooks/useDashboardData";

export const Route = createFileRoute("/")({
	component: Home,
});

function Home() {
	const { devices, usage, alerts, error, loading, toggleDevice, refetch } =
		useDashboardData();

	if (loading) {
		return (
			<main className="page-wrap flex min-h-[70vh] flex-col items-center justify-center px-4 py-12">
				<div className="flex flex-col items-center gap-3">
					<div className="h-10 w-10 animate-spin rounded-full border-4 border-[var(--line)] border-t-teal-400" />
					<p className="text-sm font-semibold text-[var(--sea-ink-soft)]">
						Loading real-time dashboard data...
					</p>
				</div>
			</main>
		);
	}

	return (
		<main className="page-wrap px-4 py-8 md:py-12">
			{/* Dashboard Header */}
			<div className="mb-8 md:mb-12 flex flex-col md:flex-row md:items-end justify-between gap-4">
				<div>
					<div className="mb-2 inline-flex items-center gap-1.5 rounded-full bg-teal-50 dark:bg-teal-950/30 px-2.5 py-0.5 text-xs font-bold text-teal-600 dark:text-teal-400 border border-teal-200/30">
						<span className="h-1.5 w-1.5 rounded-full bg-teal-500 animate-pulse" />
						Live Syncing
					</div>
					<h1 className="text-3xl font-black text-[var(--sea-ink)] tracking-tight sm:text-4xl md:text-5xl">
						Office Electricity Monitor
					</h1>
					<p className="text-sm text-[var(--sea-ink-soft)] mt-1.5 max-w-2xl">
						Real-time status, consumption levels, and anomalies breakdown across
						the building. Devices can be manually toggled via the map or the
						list.
					</p>
				</div>

				{/* Connection status/error banner */}
				{error && (
					<div className="bg-red-50 dark:bg-red-950/20 border border-red-200 dark:border-red-900/30 rounded-2xl p-3 flex items-center gap-2 text-xs font-semibold text-red-600 dark:text-red-400 max-w-sm">
						<span>⚠️</span> Connection Error: Falling back to cached state.
					</div>
				)}
			</div>

			{/* Top View Interactive Floor Plan */}
			<OfficeLayout devices={devices} onToggle={toggleDevice} />

			<div className="grid grid-cols-1 lg:grid-cols-3 gap-8 items-start">
				{/* Left Column: Device Status Panels */}
				<div className="lg:col-span-2 space-y-6">
					<h2 className="text-lg font-bold text-[var(--sea-ink-soft)] uppercase tracking-wider">
						Rooms & Devices Breakdown
					</h2>
					<DevicePanel devices={devices} onToggle={toggleDevice} />
				</div>

				{/* Right Column: Statistics & System Alerts */}
				<div className="space-y-6">
					<h2 className="text-lg font-bold text-[var(--sea-ink-soft)] uppercase tracking-wider">
						System Operations
					</h2>
					<PowerMeter usage={usage} />
					<AlertsPanel alerts={alerts} />
				</div>
			</div>
			
			<DemoControls onUpdate={refetch} />
		</main>
	);
}
