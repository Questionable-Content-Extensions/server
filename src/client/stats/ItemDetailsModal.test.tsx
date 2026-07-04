import {
    cleanup,
    fireEvent,
    render,
    screen,
    waitFor,
} from '@testing-library/react';
import type { Item } from 'models/Item';
import type { ItemList } from 'models/ItemList';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import ItemDetailsModal, { _resetItemsCache } from './ItemDetailsModal';

afterEach(() => {
    cleanup();
    _resetItemsCache();
    vi.unstubAllGlobals();
});

const ITEM: Item = {
    id: 1,
    shortName: 'Marten',
    name: 'Marten Reed',
    type: 'cast',
    color: '#aabbcc',
    first: 1,
    last: 4890,
    appearances: 3000,
    totalComics: 4890,
    presence: 61.4,
    hasImage: true,
    primaryImage: 42,
    startComicId: null,
    endComicId: null,
};

const LOCATION_ITEM: Item = {
    id: 10,
    shortName: 'Coffee shop',
    name: 'Coffee of Doom',
    type: 'location',
    color: '#334455',
    first: 5,
    last: 4800,
    appearances: 1500,
    totalComics: 4890,
    presence: 30.7,
    hasImage: false,
    primaryImage: null,
    startComicId: null,
    endComicId: null,
};

const ALL_ITEMS: ItemList[] = [
    {
        id: 2,
        shortName: 'Faye',
        name: 'Faye Whitaker',
        type: 'cast',
        color: '#112233',
        count: 2800,
        startComicId: null,
        endComicId: null,
    },
    {
        id: 10,
        shortName: 'Coffee shop',
        name: 'Coffee of Doom',
        type: 'location',
        color: '#334455',
        count: 1500,
        startComicId: null,
        endComicId: null,
    },
];

function makeFetch(responses: Record<string, unknown>) {
    // Sort longest key first so more-specific paths match before prefixes
    // (e.g. "/itemdata/1/friends" before "/itemdata/1")
    const sortedKeys = Object.keys(responses).sort(
        (a, b) => b.length - a.length,
    );
    return vi.fn((url: string) => {
        const key = sortedKeys.find((k) => url.includes(k));
        if (key === undefined) {
            return Promise.resolve({
                ok: false,
                status: 404,
                json: () => Promise.resolve(null),
            });
        }
        const data = responses[key];
        return Promise.resolve({
            ok: true,
            json: () => Promise.resolve(data),
        });
    });
}

describe('ItemDetailsModal', () => {
    beforeEach(() => {
        _resetItemsCache();
    });

    it('shows loading state while item data is fetching', () => {
        vi.stubGlobal(
            'fetch',
            vi.fn().mockReturnValue(new Promise(() => undefined)),
        );
        render(
            <ItemDetailsModal initialItemId={1} onClose={() => undefined} />,
        );
        // Both the header and body show "Loading…" while pending
        expect(screen.getAllByText('Loading…').length).toBeGreaterThan(0);
    });

    it('shows item name, type, and stats after data loads', async () => {
        vi.stubGlobal(
            'fetch',
            makeFetch({
                '/api/v3/itemdata/1': ITEM,
                '/api/v3/itemdata/1/friends': [],
                '/api/v3/itemdata/1/locations': [],
                '/api/v3/itemdata/': ALL_ITEMS,
            }),
        );
        render(
            <ItemDetailsModal initialItemId={1} onClose={() => undefined} />,
        );
        expect(await screen.findByText('Marten Reed')).toBeInTheDocument();
        expect(screen.getByText('Character')).toBeInTheDocument();
        expect(screen.getByText('3,000')).toBeInTheDocument();
        expect(screen.getByText('61.4%')).toBeInTheDocument();
    });

    it('shows the primary image when the item has one', async () => {
        vi.stubGlobal(
            'fetch',
            makeFetch({
                '/api/v3/itemdata/1': ITEM,
                '/api/v3/itemdata/1/friends': [],
                '/api/v3/itemdata/1/locations': [],
                '/api/v3/itemdata/': ALL_ITEMS,
            }),
        );
        render(
            <ItemDetailsModal initialItemId={1} onClose={() => undefined} />,
        );
        await screen.findByText('Marten Reed');
        const img = screen.getByRole('img');
        expect(img).toHaveAttribute('src', '/api/v3/itemdata/image/42');
    });

    it('shows related friends with shared count for cast items', async () => {
        vi.stubGlobal(
            'fetch',
            makeFetch({
                '/api/v3/itemdata/1': ITEM,
                '/api/v3/itemdata/1/friends': [{ id: 2, count: 1200 }],
                '/api/v3/itemdata/1/locations': [],
                '/api/v3/itemdata/': ALL_ITEMS,
            }),
        );
        render(
            <ItemDetailsModal initialItemId={1} onClose={() => undefined} />,
        );
        await screen.findByText('Marten Reed');
        await waitFor(() =>
            expect(screen.queryByText('Top co-stars')).toBeInTheDocument(),
        );
        expect(screen.getByText('Faye Whitaker')).toBeInTheDocument();
        expect(screen.getByText('1,200 shared')).toBeInTheDocument();
    });

    it('shows "Top characters" heading for location items', async () => {
        vi.stubGlobal(
            'fetch',
            makeFetch({
                '/api/v3/itemdata/10': LOCATION_ITEM,
                '/api/v3/itemdata/10/friends': [{ id: 2, count: 800 }],
                '/api/v3/itemdata/10/locations': [],
                '/api/v3/itemdata/': ALL_ITEMS,
            }),
        );
        render(
            <ItemDetailsModal initialItemId={10} onClose={() => undefined} />,
        );
        await screen.findByText('Coffee of Doom');
        await waitFor(() =>
            expect(screen.queryByText('Top characters')).toBeInTheDocument(),
        );
    });

    it('does not show image when primaryImage is null', async () => {
        vi.stubGlobal(
            'fetch',
            makeFetch({
                '/api/v3/itemdata/10': LOCATION_ITEM,
                '/api/v3/itemdata/10/friends': [],
                '/api/v3/itemdata/10/locations': [],
                '/api/v3/itemdata/': ALL_ITEMS,
            }),
        );
        render(
            <ItemDetailsModal initialItemId={10} onClose={() => undefined} />,
        );
        await screen.findByText('Coffee of Doom');
        expect(screen.queryByRole('img')).not.toBeInTheDocument();
    });

    it('shows an error message when the item fetch fails', async () => {
        vi.stubGlobal(
            'fetch',
            vi.fn().mockResolvedValue({ ok: false, status: 500 }),
        );
        render(
            <ItemDetailsModal initialItemId={1} onClose={() => undefined} />,
        );
        expect(
            await screen.findByText(/Failed to load: HTTP 500/),
        ).toBeInTheDocument();
    });

    it('calls onClose when the Close button is clicked', async () => {
        vi.stubGlobal(
            'fetch',
            makeFetch({
                '/api/v3/itemdata/1': ITEM,
                '/api/v3/itemdata/1/friends': [],
                '/api/v3/itemdata/1/locations': [],
                '/api/v3/itemdata/': ALL_ITEMS,
            }),
        );
        const onClose = vi.fn();
        render(<ItemDetailsModal initialItemId={1} onClose={onClose} />);
        await screen.findByText('Marten Reed');
        fireEvent.click(screen.getByRole('button', { name: 'Close' }));
        expect(onClose).toHaveBeenCalledOnce();
    });

    it('calls onClose when the Escape key is pressed', async () => {
        vi.stubGlobal(
            'fetch',
            makeFetch({
                '/api/v3/itemdata/1': ITEM,
                '/api/v3/itemdata/1/friends': [],
                '/api/v3/itemdata/1/locations': [],
                '/api/v3/itemdata/': ALL_ITEMS,
            }),
        );
        const onClose = vi.fn();
        render(<ItemDetailsModal initialItemId={1} onClose={onClose} />);
        await screen.findByText('Marten Reed');
        fireEvent.keyDown(document, { key: 'Escape' });
        expect(onClose).toHaveBeenCalledOnce();
    });

    it('calls onClose when the backdrop is clicked', async () => {
        vi.stubGlobal(
            'fetch',
            makeFetch({
                '/api/v3/itemdata/1': ITEM,
                '/api/v3/itemdata/1/friends': [],
                '/api/v3/itemdata/1/locations': [],
                '/api/v3/itemdata/': ALL_ITEMS,
            }),
        );
        const onClose = vi.fn();
        const { container } = render(
            <ItemDetailsModal initialItemId={1} onClose={onClose} />,
        );
        await screen.findByText('Marten Reed');
        // The backdrop is the outermost div
        fireEvent.click(container.firstChild!);
        expect(onClose).toHaveBeenCalledOnce();
    });

    it('navigates to a related item when its name is clicked', async () => {
        const fetchMock = makeFetch({
            '/api/v3/itemdata/1': ITEM,
            '/api/v3/itemdata/1/friends': [{ id: 2, count: 1200 }],
            '/api/v3/itemdata/1/locations': [],
            '/api/v3/itemdata/2': {
                ...ITEM,
                id: 2,
                shortName: 'Faye',
                name: 'Faye Whitaker',
                primaryImage: null,
            } satisfies Item,
            '/api/v3/itemdata/2/friends': [],
            '/api/v3/itemdata/2/locations': [],
            '/api/v3/itemdata/': ALL_ITEMS,
        });
        vi.stubGlobal('fetch', fetchMock);

        render(
            <ItemDetailsModal initialItemId={1} onClose={() => undefined} />,
        );
        await screen.findByText('Marten Reed');
        await waitFor(() =>
            expect(screen.queryByText('Faye Whitaker')).toBeInTheDocument(),
        );

        fireEvent.click(screen.getByRole('button', { name: 'Faye Whitaker' }));
        expect(
            await screen.findByText('Faye Whitaker', { selector: 'h3' }),
        ).toBeInTheDocument();
    });
});
