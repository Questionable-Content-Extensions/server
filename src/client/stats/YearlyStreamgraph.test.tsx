import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';

import type { YearlySpotlightResponse } from '../../../bindings/YearlySpotlightResponse';
import YearlyStreamgraph from './YearlyStreamgraph';

afterEach(cleanup);

// Three years, two characters. Chosen so that at 800 px wide (the component's
// initial width) xStep = 780 / 2 = 390 px — large enough that clientX=0
// reliably resolves to yearIdx 0 (year 2020) and clientX=780 to yearIdx 2
// (year 2022) in jsdom, where getBoundingClientRect() returns all zeros.
const RESPONSE: YearlySpotlightResponse = {
    years: [
        {
            year: 2020,
            characters: [
                { id: 1, appearances: 10 },
                { id: 2, appearances: 5 },
            ],
        },
        {
            year: 2021,
            characters: [
                { id: 1, appearances: 15 },
                { id: 2, appearances: 8 },
            ],
        },
        {
            year: 2022,
            characters: [
                { id: 1, appearances: 12 },
                { id: 2, appearances: 3 },
            ],
        },
    ],
    characters: {
        1: { name: 'Alice', color: '#ff0000' },
        2: { name: 'Bob', color: '#00ff00' },
    },
};

// Response where year 2020 has no ranked characters, so appearances default to
// 0 and the tooltip should display "not in top 5".
const SPARSE_RESPONSE: YearlySpotlightResponse = {
    years: [
        { year: 2020, characters: [] },
        { year: 2021, characters: [{ id: 1, appearances: 5 }] },
    ],
    characters: {
        1: { name: 'Alice', color: '#ff0000' },
    },
};

describe('YearlyStreamgraph', () => {
    it('renders one SVG path per character', () => {
        const { container } = render(<YearlyStreamgraph response={RESPONSE} />);
        expect(container.querySelectorAll('svg path')).toHaveLength(
            Object.keys(RESPONSE.characters).length,
        );
    });

    it('renders a year label for every year when the count fits in the 12-label budget', () => {
        render(<YearlyStreamgraph response={RESPONSE} />);
        for (const { year } of RESPONSE.years) {
            expect(screen.getByText(String(year))).toBeInTheDocument();
        }
    });

    it('renders a legend entry for each character', () => {
        render(<YearlyStreamgraph response={RESPONSE} />);
        for (const { name } of Object.values(RESPONSE.characters)) {
            expect(screen.getByText(name)).toBeInTheDocument();
        }
    });

    it('shows no tooltip before any interaction', () => {
        render(<YearlyStreamgraph response={RESPONSE} />);
        expect(
            screen.queryByTestId('streamgraph-tooltip'),
        ).not.toBeInTheDocument();
    });

    it('shows the tooltip with count on mousemove and hides it on mouseleave', () => {
        const { container } = render(<YearlyStreamgraph response={RESPONSE} />);
        // First path corresponds to character 1 (Alice); clientX=0 → yearIdx=0
        // → year 2020 → Alice has 10 appearances.
        const path = container.querySelector<SVGPathElement>('svg path')!;

        fireEvent.mouseMove(path, { clientX: 0, clientY: 50 });
        const tooltip = screen.getByTestId('streamgraph-tooltip');
        expect(tooltip).toBeInTheDocument();
        expect(tooltip.textContent).toContain('10 comics');

        fireEvent.mouseLeave(path);
        expect(
            screen.queryByTestId('streamgraph-tooltip'),
        ).not.toBeInTheDocument();
    });

    it('shows "not in top 5" when the character has no appearances in the hovered year', () => {
        const { container } = render(
            <YearlyStreamgraph response={SPARSE_RESPONSE} />,
        );
        const path = container.querySelector<SVGPathElement>('svg path')!;

        // clientX=0 → yearIdx=0 → year 2020 → Alice not in characters array → count=0
        fireEvent.mouseMove(path, { clientX: 0, clientY: 50 });
        const tooltip = screen.getByTestId('streamgraph-tooltip');
        expect(tooltip.textContent).toContain('not in top 5');
    });
});
