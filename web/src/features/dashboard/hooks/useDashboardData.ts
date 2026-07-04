import { useCallback, useEffect, useState } from "react";
import type { Device, UsageResponse } from "../types";

const API_BASE = import.meta.env.VITE_API_URL || "http://localhost:8080/api";

export function useDashboardData() {
	const [devices, setDevices] = useState<Device[]>([]);
	const [usage, setUsage] = useState<UsageResponse | null>(null);
	const [error, setError] = useState<string | null>(null);
	const [loading, setLoading] = useState<boolean>(true);

	const fetchDashboardData = useCallback(async () => {
		try {
			const [devicesRes, usageRes] = await Promise.all([
				fetch(`${API_BASE}/devices`),
				fetch(`${API_BASE}/usage`),
			]);

			if (!devicesRes.ok || !usageRes.ok) {
				throw new Error("Failed to fetch dashboard data");
			}

			const devicesData = await devicesRes.json();
			const usageData = await usageRes.json();

			setDevices(devicesData);
			setUsage(usageData);
			setError(null);
		} catch (err) {
			console.error(err);
			setError(
				err instanceof Error
					? err.message
					: "An error occurred fetching dashboard data",
			);
		} finally {
			setLoading(false);
		}
	}, []);

	const toggleDevice = async (id: string) => {
		// Optimistically toggle state in UI
		setDevices((prev) =>
			prev.map((d) =>
				d.id === id
					? { ...d, status: !d.status, last_changed: new Date().toISOString() }
					: d,
			),
		);

		try {
			const res = await fetch(`${API_BASE}/devices/${id}/toggle`, {
				method: "POST",
			});

			if (!res.ok) {
				throw new Error("Failed to toggle device");
			}

			// Fetch fresh data to align state and power metrics
			fetchDashboardData();
		} catch (err) {
			console.error(err);
			setError(err instanceof Error ? err.message : "Failed to toggle device");
			// Revert optimistic update by refetching
			fetchDashboardData();
		}
	};

	useEffect(() => {
		fetchDashboardData();
		const interval = setInterval(fetchDashboardData, 30000);
		return () => clearInterval(interval);
	}, [fetchDashboardData]);

	return {
		devices,
		usage,
		error,
		loading,
		toggleDevice,
		refetch: fetchDashboardData,
	};
}
