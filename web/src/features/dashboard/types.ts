export interface Device {
	id: string;
	name: string;
	room: string;
	device_type: "fan" | "light";
	status: boolean;
	power_consumption: number;
	last_changed: string;
}

export interface RoomBreakdown {
	room: string;
	current_watts: number;
}

export interface UsageResponse {
	total_current_watts: number;
	room_breakdown: RoomBreakdown[];
	today_kwh: number;
}
