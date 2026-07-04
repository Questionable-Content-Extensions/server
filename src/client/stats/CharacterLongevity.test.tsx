import { cleanup, render, screen } from '@testing-library/react';
import type { ItemStats } from 'models/ItemStats';
import { afterEach, describe, expect, it } from 'vitest';

import CharacterLongevity from './CharacterLongevity';

afterEach(cleanup);

const ROWS: ItemStats[] = [
    { id: 1, name: 'Alice', appearances: 50, firstComic: 1, lastComic: 500 },
    { id: 2, name: 'Bob', appearances: 30, firstComic: 10, lastComic: 200 },
    { id: 3, name: 'Carol', appearances: 10, firstComic: 5, lastComic: 1000 },
];

describe('CharacterLongevity', () => {
    it('shows loading state when sharedData is null', () => {
        render(<CharacterLongevity sharedData={null} />);
        expect(screen.getByText('Loading…')).toBeInTheDocument();
    });

    it('shows error state', () => {
        render(<CharacterLongevity sharedData={null} sharedError="HTTP 500" />);
        expect(
            screen.getByText('Failed to load data: HTTP 500'),
        ).toBeInTheDocument();
    });

    it('sorts by span descending', () => {
        render(<CharacterLongevity sharedData={ROWS} />);
        const cells = screen
            .getAllByRole('cell')
            .filter((c) =>
                ['Alice', 'Bob', 'Carol'].includes(c.textContent ?? ''),
            );
        // Carol: 1000-5=995, Alice: 500-1=499, Bob: 200-10=190
        expect(cells.map((c) => c.textContent)).toEqual([
            'Carol',
            'Alice',
            'Bob',
        ]);
    });

    it('does not mutate the input array', () => {
        const rows = [...ROWS];
        const originalOrder = rows.map((r) => r.id);
        render(<CharacterLongevity sharedData={rows} />);
        expect(rows.map((r) => r.id)).toEqual(originalOrder);
    });
});
