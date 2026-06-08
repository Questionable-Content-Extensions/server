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

interface LocationOneHitWondersProps {
    locationsData: ItemStats[] | null;
    locationsError: string | null;
}

const ONE_HIT_WONDER_THRESHOLD = 10;

export default function LocationOneHitWonders({
    locationsData,
    locationsError,
}: LocationOneHitWondersProps) {
    const [selectedItemId, setSelectedItemId] = useState<number | null>(null);
    const [sort, handleSort] = useSortState<SortKey>('appearances', 'asc');

    const wonders = useMemo(
        () =>
            locationsData?.filter(
                (r) => r.appearances <= ONE_HIT_WONDER_THRESHOLD,
            ) ?? null,
        [locationsData],
    );

    const sorted = useMemo(() => {
        if (!wonders) return null;
        const copy = [...wonders];
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
    }, [wonders, sort]);

    if (locationsError) {
        return (
            <p className="text-red-600">
                Failed to load data: {locationsError}
            </p>
        );
    }

    if (!sorted) {
        return <p className="text-gray-500">Loading…</p>;
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
                    Locations that appeared in {ONE_HIT_WONDER_THRESHOLD} or
                    fewer comics. {sorted.length} total.
                </p>
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
                                            setSelectedItemId(row.id);
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
        </>
    );
}
