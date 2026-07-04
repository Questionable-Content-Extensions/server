import type { ItemStats } from 'models/ItemStats';
import { useMemo, useState } from 'react';

import ItemDetailsModal from './ItemDetailsModal';
import {
    SortableHeader,
    StaticHeader,
    StatsTable,
    StatsTbodyRow,
    StatsTheadRow,
    comicLink,
    useSortState,
} from './StatsTable';

type SortKey = 'name' | 'span' | 'appearances' | 'firstComic' | 'lastComic';

interface CharacterLongevityProps {
    sharedData?: ItemStats[] | null;
    sharedError?: string | null;
}

export default function CharacterLongevity({
    sharedData,
    sharedError,
}: CharacterLongevityProps) {
    const [sort, handleSort] = useSortState<SortKey>('span', 'desc');
    const [selectedItemId, setSelectedItemId] = useState<number | null>(null);

    const data = useMemo(() => {
        if (!sharedData) return null;
        const copy = [...sharedData];
        copy.sort((a, b) => {
            const spanA = a.lastComic - a.firstComic;
            const spanB = b.lastComic - b.firstComic;
            const diff =
                sort.key === 'name'
                    ? a.name.localeCompare(b.name)
                    : sort.key === 'appearances'
                      ? a.appearances - b.appearances
                      : sort.key === 'firstComic'
                        ? a.firstComic - b.firstComic
                        : sort.key === 'lastComic'
                          ? a.lastComic - b.lastComic
                          : spanA - spanB;
            return sort.dir === 'asc' ? diff : -diff;
        });
        return copy;
    }, [sharedData, sort]);

    if (sharedError) {
        return (
            <p className="text-red-600">Failed to load data: {sharedError}</p>
        );
    }

    if (!data) {
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
                    Character Longevity
                </h2>
                <p className="text-sm text-gray-500 mb-4">
                    Characters ranked by the span between their first and last
                    appearance (in comic numbers). A large span means a
                    character has been part of the story for a long stretch.
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
                                sortKey="span"
                                sort={sort}
                                onSort={handleSort}
                            >
                                Span
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
                            <SortableHeader
                                sortKey="lastComic"
                                sort={sort}
                                onSort={handleSort}
                            >
                                Last comic
                            </SortableHeader>
                        </StatsTheadRow>
                    </thead>
                    <tbody>
                        {data.map((row, i) => {
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
                                    <td className="py-2 pr-4 text-right text-gray-700">
                                        {span.toLocaleString()}
                                    </td>
                                    <td className="py-2 pr-4 text-right text-gray-700">
                                        {row.appearances.toLocaleString()}
                                    </td>
                                    <td className="py-2 pr-4 text-right">
                                        <a
                                            href={comicLink(row.firstComic)}
                                            className="text-blue-600 hover:underline"
                                            target="_blank"
                                            rel="noreferrer"
                                        >
                                            #{row.firstComic}
                                        </a>
                                    </td>
                                    <td className="py-2 text-right">
                                        <a
                                            href={comicLink(row.lastComic)}
                                            className="text-blue-600 hover:underline"
                                            target="_blank"
                                            rel="noreferrer"
                                        >
                                            #{row.lastComic}
                                        </a>
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
