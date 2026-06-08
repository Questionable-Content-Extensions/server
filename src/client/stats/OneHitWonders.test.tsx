import { cleanup, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';

import type { ItemStats } from '../../../bindings/ItemStats';
import LocationOneHitWonders from './LocationOneHitWonders';
import OneHitWonders from './OneHitWonders';

afterEach(cleanup);

const CAST: ItemStats[] = [
    { id: 1, name: 'Alice', appearances: 50, firstComic: 1, lastComic: 100 },
    { id: 2, name: 'Bob', appearances: 1, firstComic: 42, lastComic: 42 },
];

const LOCATIONS: ItemStats[] = [
    {
        id: 10,
        name: 'Coffee Shop',
        appearances: 20,
        firstComic: 3,
        lastComic: 80,
    },
    { id: 11, name: 'Alley', appearances: 1, firstComic: 7, lastComic: 7 },
];

describe('OneHitWonders', () => {
    it('shows loading state when cast data is null', () => {
        render(<OneHitWonders castData={null} castError={null} />);
        expect(screen.getByText('Loading…')).toBeInTheDocument();
    });

    it('shows error state', () => {
        render(<OneHitWonders castData={null} castError="HTTP 500" />);
        expect(
            screen.getByText('Failed to load data: HTTP 500'),
        ).toBeInTheDocument();
    });

    it('shows only cast members within threshold', () => {
        render(<OneHitWonders castData={CAST} castError={null} />);
        expect(screen.getByText('Bob')).toBeInTheDocument();
        expect(screen.queryByText('Alice')).not.toBeInTheDocument();
    });

    it('shows correct total count', () => {
        render(<OneHitWonders castData={CAST} castError={null} />);
        expect(screen.getByText(/1 total/)).toBeInTheDocument();
    });
});

describe('LocationOneHitWonders', () => {
    it('shows loading state when locations data is null', () => {
        render(
            <LocationOneHitWonders
                locationsData={null}
                locationsError={null}
            />,
        );
        expect(screen.getByText('Loading…')).toBeInTheDocument();
    });

    it('shows error state', () => {
        render(
            <LocationOneHitWonders
                locationsData={null}
                locationsError="HTTP 500"
            />,
        );
        expect(
            screen.getByText('Failed to load data: HTTP 500'),
        ).toBeInTheDocument();
    });

    it('shows only locations within threshold', () => {
        render(
            <LocationOneHitWonders
                locationsData={LOCATIONS}
                locationsError={null}
            />,
        );
        expect(screen.getByText('Alley')).toBeInTheDocument();
        expect(screen.queryByText('Coffee Shop')).not.toBeInTheDocument();
    });

    it('shows correct total count', () => {
        render(
            <LocationOneHitWonders
                locationsData={LOCATIONS}
                locationsError={null}
            />,
        );
        expect(screen.getByText(/1 total/)).toBeInTheDocument();
    });
});
