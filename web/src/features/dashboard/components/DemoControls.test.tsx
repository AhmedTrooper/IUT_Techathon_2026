/**
 * @vitest-environment jsdom
 */
import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { DemoControls } from './DemoControls';

import { cleanup } from '@testing-library/react';

// Mock the API calls
global.fetch = vi.fn();

describe('DemoControls Time-Travel UI', () => {
    afterEach(() => {
        cleanup();
        vi.clearAllMocks();
    });

    it('renders the time travel buttons correctly', () => {
        render(<DemoControls onUpdate={() => {}} />);
        
        expect(screen.getByText('⏳ Time Travel (Demo)')).toBeDefined();
        expect(screen.getByText('+2 Hours')).toBeDefined();
        expect(screen.getByText('+5 Hours')).toBeDefined();
        expect(screen.getByText('Reset Time')).toBeDefined();
    });

    it('triggers the onUpdate callback when a button is clicked', async () => {
        const mockOnUpdate = vi.fn();
        
        // Mock successful fetch
        (global.fetch as any).mockResolvedValueOnce({
            ok: true,
            json: async () => ({ success: true })
        });

        render(<DemoControls onUpdate={mockOnUpdate} />);
        
        const twoHourBtn = screen.getByText('+2 Hours');
        fireEvent.click(twoHourBtn);

        // Wait for fetch to be called
        await vi.waitFor(() => {
            expect(global.fetch).toHaveBeenCalledWith(
                expect.stringContaining('/api/alerts/demo-time'),
                expect.objectContaining({ method: 'POST' })
            );
            expect(mockOnUpdate).toHaveBeenCalled();
        });
    });
});
