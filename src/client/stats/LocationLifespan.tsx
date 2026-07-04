import type { ItemStats } from 'models/ItemStats';
import { useMemo, useState } from 'react';

import ItemDetailsModal from './ItemDetailsModal';
import {
    SortableHeader,
    StaticHeader,
    StatsTable,
    StatsTbodyRow,
    StatsTheadRow,
    useSortState,
} from './StatsTable';

type SortKey = 'span' | 'appearances' | 'first' | 'last' | 'name';

interface Props {
    locationsData: ItemStats[] | null;
    locationsError: string | null;
}

export default function LocationLifespan({
    locationsData,
    locationsError,
}: Props) {
    const [sort, handleSort] = useSortState<SortKey>('span', 'desc');
    const [selectedItemId, setSelectedItemId] = useState<number | null>(null);

    const sorted = useMemo(() => {
        if (!locationsData) return null;
        const copy = [...locationsData];
        copy.sort((a, b) => {
            const spanA = a.lastComic - a.firstComic;
            const spanB = b.lastComic - b.firstComic;
            const diff =
                sort.key === 'span'
                    ? spanA - spanB
                    : sort.key === 'appearances'
                      ? a.appearances - b.appearances
                      : sort.key === 'first'
                        ? a.firstComic - b.firstComic
                        : sort.key === 'last'
                          ? a.lastComic - b.lastComic
                          : a.name.localeCompare(b.name);
            return sort.dir === 'asc' ? diff : -diff;
        });
        return copy;
    }, [locationsData, sort]);

    if (locationsError)
        return (
            <p className="text-red-600">
                Failed to load data: {locationsError}
            </p>
        );
    if (!sorted) return <p className="text-gray-500">Loading…</p>;

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
                    Location Lifespan
                </h2>
                <p className="text-sm text-gray-500 mb-4">
                    Every location ranked by how long it has been in use,
                    measured as the comic-number span between its first and last
                    appearance. A wide span with few appearances indicates a
                    location used only occasionally over a long period.
                </p>
                <StatsTable>
                    <thead>
                        <StatsTheadRow>
                            <StaticHeader className="w-12">#</StaticHeader>
                            <SortableHeader
                                sortKey="name"
                                sort={sort}
                                onSort={handleSort}
                                align="left"
                            >
                                Name
                            </SortableHeader>
                            <SortableHeader
                                sortKey="first"
                                sort={sort}
                                onSort={handleSort}
                            >
                                First comic
                            </SortableHeader>
                            <SortableHeader
                                sortKey="last"
                                sort={sort}
                                onSort={handleSort}
                            >
                                Last comic
                            </SortableHeader>
                            <SortableHeader
                                sortKey="span"
                                sort={sort}
                                onSort={handleSort}
                            >
                                Comic span
                            </SortableHeader>
                            <SortableHeader
                                sortKey="appearances"
                                sort={sort}
                                onSort={handleSort}
                            >
                                Appearances
                            </SortableHeader>
                        </StatsTheadRow>
                    </thead>
                    <tbody>
                        {sorted.map((row, i) => {
                            const span = row.lastComic - row.firstComic;
                            return (
                                <StatsTbodyRow key={row.id}>
                                    <td className="py-2 pr-4 text-gray-400">
                                        {i + 1}
                                    </td>
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
                                    <td className="py-2 pr-4 text-right text-gray-500">
                                        #{row.firstComic}
                                    </td>
                                    <td className="py-2 pr-4 text-right text-gray-500">
                                        #{row.lastComic}
                                    </td>
                                    <td className="py-2 pr-4 text-right font-medium text-indigo-700">
                                        {span.toLocaleString()}
                                    </td>
                                    <td className="py-2 text-right text-gray-500">
                                        {row.appearances.toLocaleString()}
                                    </td>
                                </StatsTbodyRow>
                            );
                        })}
                    </tbody>
                </StatsTable>
            </div>
        </>
    );
}
