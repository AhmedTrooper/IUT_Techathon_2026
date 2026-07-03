// @vitest-environment jsdom
import { render, screen, fireEvent, cleanup } from "@testing-library/react";
import { describe, it, expect, vi, afterEach } from "vitest";
import { DevicePanel } from "./DevicePanel";
import type { Device } from "../../types";

afterEach(() => {
	cleanup();
});

const MOCK_DEVICES: Device[] = [
	{
		id: "drawing_room_fan_1",
		name: "Fan 1",
		device_type: "fan",
		room: "drawing_room",
		status: true, // ON
		power_consumption: 60,
		last_changed: new Date().toISOString(),
	},
];

describe("DevicePanel Dashboard Component", () => {
	it("renders devices and correct toggle states", () => {
		const handleToggle = vi.fn();
		render(<DevicePanel devices={MOCK_DEVICES} onToggle={handleToggle} />);
		
		// Should render the room title
		expect(screen.getByText("Drawing Room")).toBeDefined();
		// Should render the device name
		expect(screen.getByText("Fan 1")).toBeDefined();
		// Should show active power consumption
		expect(screen.getByText("60W")).toBeDefined();
		
		// Because it's ON, button should say 'Turn OFF'
		const toggleBtn = screen.getByText("Turn OFF");
		expect(toggleBtn).toBeDefined();

		// Clicking the button should call the toggle prop
		fireEvent.click(toggleBtn);
		expect(handleToggle).toHaveBeenCalledWith("drawing_room_fan_1");
	});
});
