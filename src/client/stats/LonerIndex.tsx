import { useEffect, useMemo, useState } from 'react';

import type { LonerEntry } from '../../../bindings/LonerEntry';
import ItemDetailsModal from './ItemDetailsModal';
import {
    SortableHeader,
    StaticHeader,
    StatsTable,
    StatsTbodyRow,
    StatsTheadRow,
    useSortState,
} from './StatsTable';

type SortKey = 'avg' | 'appearances' | 'name';

export default function LonerIndex() {
    const [data, setData] = useState<LonerEntry[] | null>(null);
    const [error, setError] = useState<string | null>(null);
    const [sort, handleSort] = useSortState<SortKey>('avg', 'asc');
    const [selectedItemId, setSelectedItemId] = useState<number | null>(null);

    useEffect(() => {
        fetch('/api/v3/stats/loner-index')
            .then((r) => {
                if (!r.ok) throw new Error(`HTTP ${r.status}`);
                return r.json() as Promise<LonerEntry[]>;
            })
            .then(setData)
            .catch((e: unknown) =>
                setError(e instanceof Error ? e.message : String(e)),
            );
    }, []);

    const sorted = useMemo(() => {
        if (!data) return null;
        const copy = [...data];
        copy.sort((a, b) => {
            const diff =
                sort.key === 'avg'
                    ? a.avgCoCast - b.avgCoCast
                    : sort.key === 'appearances'
                      ? a.appearances - b.appearances
                      : a.name.localeCompare(b.name);
            return sort.dir === 'asc' ? diff : -diff;
        });
        return copy;
    }, [data, sort]);

    if (error)
        return <p className="text-red-600">Failed to load data: {error}</p>;
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
                    Loner Index
                </h2>
                <p className="text-sm text-gray-500 mb-4">
                    Characters sorted by how few co-stars they typically appear
                    with. The average co-cast size measures how many other cast
                    members share a comic with this character on average
                    (excluding themselves). A low value means the character
                    tends to appear alone or in very small scenes. Only
                    characters with at least 10 appearances are shown.
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
                                sortKey="appearances"
                                sort={sort}
                                onSort={handleSort}
                            >
                                Appearances
                            </SortableHeader>
                            <SortableHeader
                                sortKey="avg"
                                sort={sort}
                                onSort={handleSort}
                            >
                                Avg co-cast size
                            </SortableHeader>
                        </StatsTheadRow>
                    </thead>
                    <tbody>
                        {sorted.map((row, i) => (
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
                                    {row.appearances.toLocaleString()}
                                </td>
                                <td
                                    className={`py-2 text-right font-medium ${row.avgCoCast < 0.5 ? 'text-blue-600' : row.avgCoCast < 1.5 ? 'text-indigo-700' : 'text-gray-600'}`}
                                >
                                    {row.avgCoCast.toFixed(2)}
                                </td>
                            </StatsTbodyRow>
                        ))}
                    </tbody>
                </StatsTable>
            </div>
        </>
    );
}
