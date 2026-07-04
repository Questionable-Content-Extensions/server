import type { ItemStats } from 'models/ItemStats';
import { useEffect, useMemo, useState } from 'react';

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

type ItemSortKey = 'name' | 'appearances' | 'firstComic' | 'lastComic';

interface ItemStatsTableProps {
    endpoint: string;
    title: string;
    description: string;
    sortBy: 'appearances' | 'firstComic';
    sharedData?: ItemStats[] | null;
    sharedError?: string | null;
}

export default function ItemStatsTable({
    endpoint,
    title,
    description,
    sortBy,
    sharedData,
    sharedError,
}: ItemStatsTableProps) {
    const [localData, setLocalData] = useState<ItemStats[] | null>(null);
    const [localError, setLocalError] = useState<string | null>(null);
    const [selectedItemId, setSelectedItemId] = useState<number | null>(null);
    const initialDir = sortBy === 'firstComic' ? 'asc' : 'desc';
    const [sort, handleSort] = useSortState<ItemSortKey>(sortBy, initialDir);

    useEffect(() => {
        if (sharedData !== undefined || sharedError !== undefined) return;
        fetch(endpoint)
            .then((r) => {
                if (!r.ok) throw new Error(`HTTP ${r.status}`);
                return r.json() as Promise<ItemStats[]>;
            })
            .then((rows) => setLocalData(rows))
            .catch((e: unknown) =>
                setLocalError(e instanceof Error ? e.message : String(e)),
            );
    }, [endpoint, sharedData, sharedError]);

    const rawData = sharedData !== undefined ? sharedData : localData;
    const error = sharedError !== undefined ? sharedError : localError;

    const data = useMemo(() => {
        if (!rawData) return null;
        const copy = [...rawData];
        copy.sort((a, b) => {
            const diff =
                sort.key === 'name'
                    ? a.name.localeCompare(b.name)
                    : sort.key === 'appearances'
                      ? a.appearances - b.appearances
                      : sort.key === 'lastComic'
                        ? a.lastComic - b.lastComic
                        : a.firstComic - b.firstComic;
            return sort.dir === 'asc' ? diff : -diff;
        });
        return copy;
    }, [rawData, sort]);

    if (error) {
        return <p className="text-red-600">Failed to load data: {error}</p>;
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
                    {title}
                </h2>
                <p className="text-sm text-gray-500 mb-4">{description}</p>
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
                        {data.map((row, i) => (
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
                        ))}
                    </tbody>
                </StatsTable>
            </div>
        </>
    );
}
