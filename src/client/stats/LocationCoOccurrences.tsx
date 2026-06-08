import { useEffect, useMemo, useState } from 'react';

import type { LocationCoOccurrenceResponse } from '../../../bindings/LocationCoOccurrenceResponse';
import ItemDetailsModal from './ItemDetailsModal';
import {
    SortableHeader,
    StaticHeader,
    StatsTable,
    StatsTbodyRow,
    StatsTheadRow,
    useSortState,
} from './StatsTable';

type SortKey = 'loc1' | 'loc2' | 'together' | 'pctL1' | 'pctL2';

function pct(together: number, appearances: number) {
    if (appearances === 0) return '—';
    return `${((together / appearances) * 100).toFixed(1)}%`;
}

export default function LocationCoOccurrences() {
    const [data, setData] = useState<LocationCoOccurrenceResponse | null>(null);
    const [error, setError] = useState<string | null>(null);
    const [sort, handleSort] = useSortState<SortKey>('together', 'desc');
    const [selectedItemId, setSelectedItemId] = useState<number | null>(null);

    useEffect(() => {
        fetch('/api/v3/stats/location-co-occurrences')
            .then((r) => {
                if (!r.ok) throw new Error(`HTTP ${r.status}`);
                return r.json() as Promise<LocationCoOccurrenceResponse>;
            })
            .then(setData)
            .catch((e: unknown) =>
                setError(e instanceof Error ? e.message : String(e)),
            );
    }, []);

    const sortedPairs = useMemo(() => {
        if (!data) return null;
        return [...data.pairs].sort((a, b) => {
            const l1a = data.locations[a.location1Id];
            const l2a = data.locations[a.location2Id];
            const l1b = data.locations[b.location1Id];
            const l2b = data.locations[b.location2Id];
            let diff: number;
            if (sort.key === 'loc1') {
                diff = (l1a?.name ?? '').localeCompare(l1b?.name ?? '');
            } else if (sort.key === 'loc2') {
                diff = (l2a?.name ?? '').localeCompare(l2b?.name ?? '');
            } else if (sort.key === 'pctL1') {
                const pA = l1a ? a.comicsTogether / l1a.appearances : 0;
                const pB = l1b ? b.comicsTogether / l1b.appearances : 0;
                diff = pA - pB;
            } else if (sort.key === 'pctL2') {
                const pA = l2a ? a.comicsTogether / l2a.appearances : 0;
                const pB = l2b ? b.comicsTogether / l2b.appearances : 0;
                diff = pA - pB;
            } else {
                diff = a.comicsTogether - b.comicsTogether;
            }
            return sort.dir === 'asc' ? diff : -diff;
        });
    }, [data, sort]);

    if (error) {
        return <p className="text-red-600">Failed to load data: {error}</p>;
    }

    if (!sortedPairs || !data) {
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
                    Location Co-Occurrences
                </h2>
                <p className="text-sm text-gray-500 mb-4">
                    The top 50 location pairs by how often they appear in the
                    same comic. The % columns show what share of each location's
                    total appearances include the other.
                </p>
                <StatsTable>
                    <thead>
                        <StatsTheadRow>
                            <StaticHeader className="w-12">#</StaticHeader>
                            <SortableHeader
                                sortKey="loc1"
                                sort={sort}
                                onSort={handleSort}
                                align="left"
                            >
                                Location 1
                            </SortableHeader>
                            <SortableHeader
                                sortKey="loc2"
                                sort={sort}
                                onSort={handleSort}
                                align="left"
                            >
                                Location 2
                            </SortableHeader>
                            <SortableHeader
                                sortKey="together"
                                sort={sort}
                                onSort={handleSort}
                            >
                                Comics together
                            </SortableHeader>
                            <SortableHeader
                                sortKey="pctL1"
                                sort={sort}
                                onSort={handleSort}
                            >
                                % of L1
                            </SortableHeader>
                            <SortableHeader
                                sortKey="pctL2"
                                sort={sort}
                                onSort={handleSort}
                            >
                                % of L2
                            </SortableHeader>
                        </StatsTheadRow>
                    </thead>
                    <tbody>
                        {sortedPairs.map((pair, i) => {
                            const l1 = data.locations[pair.location1Id];
                            const l2 = data.locations[pair.location2Id];
                            return (
                                <StatsTbodyRow
                                    key={`${pair.location1Id}-${pair.location2Id}`}
                                >
                                    <td className="py-2 pr-4 text-gray-400">
                                        {i + 1}
                                    </td>
                                    <td className="py-2 pr-4">
                                        <button
                                            type="button"
                                            onClick={() => {
                                                setSelectedItemId(
                                                    pair.location1Id,
                                                );
                                            }}
                                            className="font-medium text-gray-900 hover:text-blue-600 hover:underline text-left"
                                        >
                                            {l1?.name}
                                        </button>
                                    </td>
                                    <td className="py-2 pr-4">
                                        <button
                                            type="button"
                                            onClick={() => {
                                                setSelectedItemId(
                                                    pair.location2Id,
                                                );
                                            }}
                                            className="font-medium text-gray-900 hover:text-blue-600 hover:underline text-left"
                                        >
                                            {l2?.name}
                                        </button>
                                    </td>
                                    <td className="py-2 pr-4 text-right text-gray-700">
                                        {pair.comicsTogether.toLocaleString()}
                                    </td>
                                    <td className="py-2 pr-4 text-right text-gray-500">
                                        {l1
                                            ? pct(
                                                  pair.comicsTogether,
                                                  l1.appearances,
                                              )
                                            : '—'}
                                    </td>
                                    <td className="py-2 text-right text-gray-500">
                                        {l2
                                            ? pct(
                                                  pair.comicsTogether,
                                                  l2.appearances,
                                              )
                                            : '—'}
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
