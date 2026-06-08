import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';

import type { ItemStats } from '../../../bindings/ItemStats';
import { _resetItemsCache } from './ItemDetailsModal';
import ItemStatsTable from './ItemStatsTable';

afterEach(() => {
    cleanup();
    _resetItemsCache();
    vi.unstubAllGlobals();
});

const ROWS: ItemStats[] = [
    { id: 1, name: 'Alice', appearances: 50, firstComic: 3, lastComic: 100 },
    { id: 2, name: 'Bob', appearances: 30, firstComic: 1, lastComic: 80 },
    { id: 3, name: 'Carol', appearances: 10, firstComic: 5, lastComic: 60 },
];

describe('ItemStatsTable with sharedData', () => {
    it('renders the table from sharedData without making a fetch', () => {
        const fetchSpy = vi.spyOn(globalThis, 'fetch');

        render(
            <ItemStatsTable
                endpoint="/api/v3/stats/cast"
                title="Character Rankings"
                description="Ranked by appearances."
                sortBy="appearances"
                sharedData={ROWS}
            />,
        );

        expect(screen.getByText('Alice')).toBeInTheDocument();
        expect(screen.getByText('Bob')).toBeInTheDocument();
        expect(fetchSpy).not.toHaveBeenCalled();

        fetchSpy.mockRestore();
    });

    it('shows the loading state when sharedData is null', () => {
        render(
            <ItemStatsTable
                endpoint="/api/v3/stats/cast"
                title="Character Rankings"
                description="Ranked by appearances."
                sortBy="appearances"
                sharedData={null}
            />,
        );

        expect(screen.getByText('Loading…')).toBeInTheDocument();
    });

    it('shows the error when sharedError is provided', () => {
        render(
            <ItemStatsTable
                endpoint="/api/v3/stats/cast"
                title="Character Rankings"
                description="Ranked by appearances."
                sortBy="appearances"
                sharedData={null}
                sharedError="HTTP 500"
            />,
        );

        expect(
            screen.getByText('Failed to load data: HTTP 500'),
        ).toBeInTheDocument();
    });

    it('sorts rows by firstComic when sortBy="firstComic"', () => {
        render(
            <ItemStatsTable
                endpoint="/api/v3/stats/cast"
                title="Character Debuts"
                description="By first appearance."
                sortBy="firstComic"
                sharedData={ROWS}
            />,
        );

        const cells = screen
            .getAllByRole('cell')
            .filter((c) =>
                ['Alice', 'Bob', 'Carol'].includes(c.textContent ?? ''),
            );
        expect(cells.map((c) => c.textContent)).toEqual([
            'Bob', // firstComic: 1
            'Alice', // firstComic: 3
            'Carol', // firstComic: 5
        ]);
    });

    it('does not mutate the sharedData array when sorting', () => {
        const rows = [...ROWS];
        const originalOrder = rows.map((r) => r.id);

        render(
            <ItemStatsTable
                endpoint="/api/v3/stats/cast"
                title="Character Debuts"
                description="By first appearance."
                sortBy="firstComic"
                sharedData={rows}
            />,
        );

        expect(rows.map((r) => r.id)).toEqual(originalOrder);
    });

    it('sorts rows by name descending when name header is clicked (defaultDir=desc)', () => {
        render(
            <ItemStatsTable
                endpoint="/api/v3/stats/cast"
                title="Character Rankings"
                description="Ranked by appearances."
                sortBy="appearances"
                sharedData={ROWS}
            />,
        );

        fireEvent.click(screen.getByRole('columnheader', { name: /Name/i }));

        const cells = screen
            .getAllByRole('cell')
            .filter((c) =>
                ['Alice', 'Bob', 'Carol'].includes(c.textContent ?? ''),
            );
        expect(cells.map((c) => c.textContent)).toEqual([
            'Carol',
            'Bob',
            'Alice',
        ]);
    });

    it('sorts rows by name ascending when name header is clicked twice', () => {
        render(
            <ItemStatsTable
                endpoint="/api/v3/stats/cast"
                title="Character Rankings"
                description="Ranked by appearances."
                sortBy="appearances"
                sharedData={ROWS}
            />,
        );

        const nameHeader = screen.getByRole('columnheader', { name: /Name/i });
        fireEvent.click(nameHeader);
        fireEvent.click(nameHeader);

        const cells = screen
            .getAllByRole('cell')
            .filter((c) =>
                ['Alice', 'Bob', 'Carol'].includes(c.textContent ?? ''),
            );
        expect(cells.map((c) => c.textContent)).toEqual([
            'Alice',
            'Bob',
            'Carol',
        ]);
    });

    it('sorts rows by lastComic descending when last comic header is clicked', () => {
        render(
            <ItemStatsTable
                endpoint="/api/v3/stats/cast"
                title="Character Rankings"
                description="Ranked by appearances."
                sortBy="appearances"
                sharedData={ROWS}
            />,
        );

        fireEvent.click(
            screen.getByRole('columnheader', { name: /Last comic/i }),
        );

        const cells = screen
            .getAllByRole('cell')
            .filter((c) =>
                ['Alice', 'Bob', 'Carol'].includes(c.textContent ?? ''),
            );
        expect(cells.map((c) => c.textContent)).toEqual([
            'Alice', // lastComic: 100
            'Bob', // lastComic: 80
            'Carol', // lastComic: 60
        ]);
    });

    it('opens the item details modal when a name is clicked', () => {
        vi.stubGlobal(
            'fetch',
            vi.fn().mockReturnValue(new Promise(() => undefined)),
        );

        render(
            <ItemStatsTable
                endpoint="/api/v3/stats/cast"
                title="Character Rankings"
                description="Ranked by appearances."
                sortBy="appearances"
                sharedData={ROWS}
            />,
        );

        fireEvent.click(screen.getByRole('button', { name: 'Alice' }));
        // The modal header shows "Loading…" as a heading
        expect(
            screen.getByRole('heading', { name: 'Loading…' }),
        ).toBeInTheDocument();
    });
});
