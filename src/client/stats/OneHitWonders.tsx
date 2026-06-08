import { useMemo, useState } from 'react';

import type { ItemStats } from '../../../bindings/ItemStats';
import ItemDetailsModal from './ItemDetailsModal';
import {
    SortableHeader,
    StatsTable,
    StatsTbodyRow,
    StatsTheadRow,
    comicLink,
    useSortState,
} from './StatsTable';

type SortKey = 'name' | 'appearances' | 'firstComic';

interface SectionProps {
    title: string;
    items: ItemStats[];
    onItemClick: (id: number) => void;
}

function OneHitSection({ title, items, onItemClick }: SectionProps) {
    const [sort, handleSort] = useSortState<SortKey>('appearances', 'asc');

    const sorted = useMemo(() => {
        const copy = [...items];
        copy.sort((a, b) => {
            const diff =
                sort.key === 'name'
                    ? a.name.localeCompare(b.name)
                    : sort.key === 'firstComic'
                      ? a.firstComic - b.firstComic
                      : a.appearances - b.appearances;
            return sort.dir === 'asc' ? diff : -diff;
        });
        return copy;
    }, [items, sort]);

    if (items.length === 0) return null;
    return (
        <div className="mb-8">
            <h3 className="text-lg font-semibold text-gray-700 mb-3">
                {title}
            </h3>
            <StatsTable>
                <thead>
                    <StatsTheadRow>
                        <SortableHeader
                            sortKey="name"
                            sort={sort}
                            onSort={handleSort}
                            align="left"
                        >
                            Name
                        </SortableHeader>
                        <SortableHeader
                            sortKey="appearances"
                            sort={sort}
                            onSort={handleSort}
                        >
                            Appearances
                        </SortableHeader>
                        <SortableHeader
                            sortKey="firstComic"
                            sort={sort}
                            onSort={handleSort}
                        >
                            First comic
                        </SortableHeader>
                    </StatsTheadRow>
                </thead>
                <tbody>
                    {sorted.map((row) => (
                        <StatsTbodyRow key={row.id}>
                            <td className="py-2 pr-4">
                                <button
                                    type="button"
                                    onClick={() => {
                                        onItemClick(row.id);
                                    }}
                                    className="font-medium text-gray-900 hover:text-blue-600 hover:underline text-left"
                                >
                                    {row.name}
                                </button>
                            </td>
                            <td className="py-2 pr-4 text-right text-gray-700">
                                {row.appearances}
                            </td>
                            <td className="py-2 text-right">
                                <a
                                    href={comicLink(row.firstComic)}
                                    className="text-blue-600 hover:underline"
                                    target="_blank"
                                    rel="noreferrer"
                                >
                                    #{row.firstComic}
                                </a>
                            </td>
                        </StatsTbodyRow>
                    ))}
                </tbody>
            </StatsTable>
        </div>
    );
}

interface OneHitWondersProps {
    castData: ItemStats[] | null;
    castError: string | null;
    locationsData: ItemStats[] | null;
    locationsError: string | null;
}

const ONE_HIT_WONDER_THRESHOLD = 10;
export default function OneHitWonders({
    castData,
    castError,
    locationsData,
    locationsError,
}: OneHitWondersProps) {
    const [selectedItemId, setSelectedItemId] = useState<number | null>(null);
    const castWonders = useMemo(
        () =>
            castData?.filter(
                (r) => r.appearances <= ONE_HIT_WONDER_THRESHOLD,
            ) ?? null,
        [castData],
    );
    const locationWonders = useMemo(
        () =>
            locationsData?.filter(
                (r) => r.appearances <= ONE_HIT_WONDER_THRESHOLD,
            ) ?? null,
        [locationsData],
    );

    const error = castError ?? locationsError;
    if (error) {
        return <p className="text-red-600">Failed to load data: {error}</p>;
    }

    if (!castWonders || !locationWonders) {
        return <p className="text-gray-500">Loading…</p>;
    }

    const total = castWonders.length + locationWonders.length;

    function handleItemClick(id: number) {
        setSelectedItemId(id);
    }

    return (
        <>
            {selectedItemId !== null && (
                <ItemDetailsModal
                    initialItemId={selectedItemId}
                    onClose={() => {
                        setSelectedItemId(null);
                    }}
                />
            )}
            <div>
                <h2 className="text-xl font-semibold text-gray-800 mb-1">
                    One-Hit Wonders
                </h2>
                <p className="text-sm text-gray-500 mb-4">
                    Characters and locations that appeared in{' '}
                    {ONE_HIT_WONDER_THRESHOLD} or fewer comics. {total} total.
                </p>
                <OneHitSection
                    title={`Characters (${castWonders.length})`}
                    items={castWonders}
                    onItemClick={handleItemClick}
                />
                <OneHitSection
                    title={`Locations (${locationWonders.length})`}
                    items={locationWonders}
                    onItemClick={handleItemClick}
                />
            </div>
        </>
    );
}
