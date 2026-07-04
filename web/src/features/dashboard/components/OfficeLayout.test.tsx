// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { Device } from "../../types";
import { OfficeLayout } from "./OfficeLayout";

afterEach(() => {
	cleanup();
});

const MOCK_DEVICES: Device[] = [
	{
		id: "drawing_room_light_1",
		name: "Light 1",
		device_type: "light",
		room: "drawing_room",
		status: true,
		power_consumption: 15,
		last_changed: new Date().toISOString(),
	},
	{
		id: "drawing_room_fan_1",
		name: "Fan 1",
		device_type: "fan",
		room: "drawing_room",
		status: false,
		power_consumption: 60,
		last_changed: new Date().toISOString(),
	},
];

describe("OfficeLayout Dashboard Component", () => {
	it("should properly render the 3 required rooms", () => {
		render(<OfficeLayout devices={MOCK_DEVICES} onToggle={vi.fn()} />);

		expect(screen.getByText("Drawing Room")).toBeDefined();
		expect(screen.getByText("Work Room 1")).toBeDefined();
		expect(screen.getByText("Work Room 2")).toBeDefined();
		expect(screen.getByText("ENTRY")).toBeDefined();
	});

	it("should map interactive device nodes correctly and trigger onToggle", () => {
		const handleToggle = vi.fn();
		render(<OfficeLayout devices={MOCK_DEVICES} onToggle={handleToggle} />);

		// The active light should be rendered
		const lightButton = screen.getByTitle("Light 1 (ON)");
		expect(lightButton).toBeDefined();

		// The inactive fan should be rendered
		const fanButton = screen.getByTitle("Fan 1 (OFF)");
		expect(fanButton).toBeDefined();

		// Clicking the device icon should trigger the toggle API handler
		fireEvent.click(lightButton);
		expect(handleToggle).toHaveBeenCalledWith("drawing_room_light_1");

		fireEvent.click(fanButton);
		expect(handleToggle).toHaveBeenCalledWith("drawing_room_fan_1");
	});

	it("should visually apply glowing styles to active devices only", () => {
		render(<OfficeLayout devices={MOCK_DEVICES} onToggle={vi.fn()} />);

		const lightButton = screen.getByTitle("Light 1 (ON)");
		// When ON, it should have the glowing teal/amber shadow classes
		expect(lightButton.className).toContain(
			"shadow-[0_0_25px_rgba(251,191,36,0.6)]",
		);

		const fanButton = screen.getByTitle("Fan 1 (OFF)");
		// When OFF, it should have generic styling
		expect(fanButton.className).toContain("text-gray-500");
	});
});
