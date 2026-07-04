import { cleanup, render, screen } from '@testing-library/react';
import type { ComebackCharacter } from 'models/ComebackCharacter';
import { afterEach, describe, expect, it, vi } from 'vitest';

import ComebackCharacters from './ComebackCharacters';

afterEach(cleanup);

const DATA: ComebackCharacter[] = [
    {
        id: 1,
        name: 'Alice',
        lastComic: 100,
        returnComic: 600,
        gapDays: 1095,
    },
    {
        id: 2,
        name: 'Bob',
        lastComic: 200,
        returnComic: 300,
        gapDays: 120,
    },
];

describe('ComebackCharacters', () => {
    it('shows loading state', () => {
        vi.stubGlobal(
            'fetch',
            vi.fn().mockReturnValue(new Promise(() => undefined)),
        );
        render(<ComebackCharacters />);
        expect(screen.getByText('Loading…')).toBeInTheDocument();
        vi.unstubAllGlobals();
    });

    it('renders character names and gap when data loads', async () => {
        vi.stubGlobal(
            'fetch',
            vi.fn().mockResolvedValue({
                ok: true,
                json: () => Promise.resolve(DATA),
            }),
        );
        render(<ComebackCharacters />);
        expect(await screen.findByText('Alice')).toBeInTheDocument();
        expect(screen.getByText('Bob')).toBeInTheDocument();
        vi.unstubAllGlobals();
    });

    it('formats gaps in years for large gaps', async () => {
        vi.stubGlobal(
            'fetch',
            vi.fn().mockResolvedValue({
                ok: true,
                json: () => Promise.resolve(DATA),
            }),
        );
        render(<ComebackCharacters />);
        await screen.findByText('Alice');
        expect(screen.getByText(/3\.0y/)).toBeInTheDocument();
        vi.unstubAllGlobals();
    });

    it('shows error message on fetch failure', async () => {
        vi.stubGlobal(
            'fetch',
            vi.fn().mockResolvedValue({ ok: false, status: 500 }),
        );
        render(<ComebackCharacters />);
        expect(
            await screen.findByText('Failed to load data: HTTP 500'),
        ).toBeInTheDocument();
        vi.unstubAllGlobals();
    });
});
