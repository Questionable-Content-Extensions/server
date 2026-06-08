import { useEffect, useMemo, useState } from 'react';

import type { CoAppearancesResponse } from '../../../bindings/CoAppearancesResponse';
import ItemDetailsModal from './ItemDetailsModal';
import {
    SortableHeader,
    StaticHeader,
    StatsTable,
    StatsTbodyRow,
    StatsTheadRow,
    useSortState,
} from './StatsTable';

type SortKey = 'char1' | 'char2' | 'together' | 'pctC1' | 'pctC2';

function pct(together: number, appearances: number) {
    if (appearances === 0) return '—';
    return `${((together / appearances) * 100).toFixed(1)}%`;
}

export default function CoAppearances() {
    const [data, setData] = useState<CoAppearancesResponse | null>(null);
    const [error, setError] = useState<string | null>(null);
    const [sort, handleSort] = useSortState<SortKey>('together', 'desc');
    const [selectedItemId, setSelectedItemId] = useState<number | null>(null);

    useEffect(() => {
        fetch('/api/v3/stats/co-appearances')
            .then((r) => {
                if (!r.ok) throw new Error(`HTTP ${r.status}`);
                return r.json() as Promise<CoAppearancesResponse>;
            })
            .then(setData)
            .catch((e: unknown) =>
                setError(e instanceof Error ? e.message : String(e)),
            );
    }, []);

    const sortedPairs = useMemo(() => {
        if (!data) return null;
        return [...data.pairs].sort((a, b) => {
            const c1a = data.characters[a.character1Id];
            const c2a = data.characters[a.character2Id];
            const c1b = data.characters[b.character1Id];
            const c2b = data.characters[b.character2Id];
            let diff: number;
            if (sort.key === 'char1') {
                diff = (c1a?.name ?? '').localeCompare(c1b?.name ?? '');
            } else if (sort.key === 'char2') {
                diff = (c2a?.name ?? '').localeCompare(c2b?.name ?? '');
            } else if (sort.key === 'pctC1') {
                const pA = c1a ? a.comicsTogether / c1a.appearances : 0;
                const pB = c1b ? b.comicsTogether / c1b.appearances : 0;
                diff = pA - pB;
            } else if (sort.key === 'pctC2') {
                const pA = c2a ? a.comicsTogether / c2a.appearances : 0;
                const pB = c2b ? b.comicsTogether / c2b.appearances : 0;
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
                    Who Appears Together
                </h2>
                <p className="text-sm text-gray-500 mb-4">
                    The top 100 character pairs by shared comic appearances. The
                    % columns show what share of each character's total
                    appearances include the other.
                </p>
                <StatsTable>
                    <thead>
                        <StatsTheadRow>
                            <StaticHeader className="w-12">#</StaticHeader>
                            <SortableHeader
                                sortKey="char1"
                                sort={sort}
                                onSort={handleSort}
                                align="left"
                            >
                                Character 1
                            </SortableHeader>
                            <SortableHeader
                                sortKey="char2"
                                sort={sort}
                                onSort={handleSort}
                                align="left"
                            >
                                Character 2
                            </SortableHeader>
                            <SortableHeader
                                sortKey="together"
                                sort={sort}
                                onSort={handleSort}
                            >
                                Comics together
                            </SortableHeader>
                            <SortableHeader
                                sortKey="pctC1"
                                sort={sort}
                                onSort={handleSort}
                            >
                                % of C1
                            </SortableHeader>
                            <SortableHeader
                                sortKey="pctC2"
                                sort={sort}
                                onSort={handleSort}
                            >
                                % of C2
                            </SortableHeader>
                        </StatsTheadRow>
                    </thead>
                    <tbody>
                        {sortedPairs.map((pair, i) => {
                            const c1 = data.characters[pair.character1Id];
                            const c2 = data.characters[pair.character2Id];
                            return (
                                <StatsTbodyRow
                                    key={`${pair.character1Id}-${pair.character2Id}`}
                                >
                                    <td className="py-2 pr-4 text-gray-400">
                                        {i + 1}
                                    </td>
                                    <td className="py-2 pr-4">
                                        <button
                                            type="button"
                                            onClick={() => {
                                                setSelectedItemId(
                                                    pair.character1Id,
                                                );
                                            }}
                                            className="font-medium text-gray-900 hover:text-blue-600 hover:underline text-left"
                                        >
                                            {c1?.name}
                                        </button>
                                    </td>
                                    <td className="py-2 pr-4">
                                        <button
                                            type="button"
                                            onClick={() => {
                                                setSelectedItemId(
                                                    pair.character2Id,
                                                );
                                            }}
                                            className="font-medium text-gray-900 hover:text-blue-600 hover:underline text-left"
                                        >
                                            {c2?.name}
                                        </button>
                                    </td>
                                    <td className="py-2 pr-4 text-right text-gray-700">
                                        {pair.comicsTogether.toLocaleString()}
                                    </td>
                                    <td className="py-2 pr-4 text-right text-gray-500">
                                        {c1
                                            ? pct(
                                                  pair.comicsTogether,
                                                  c1.appearances,
                                              )
                                            : '—'}
                                    </td>
                                    <td className="py-2 text-right text-gray-500">
                                        {c2
                                            ? pct(
                                                  pair.comicsTogether,
                                                  c2.appearances,
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
