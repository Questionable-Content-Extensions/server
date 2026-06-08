import { useEffect, useMemo, useState } from 'react';

import type { BestFriendResponse } from '../../../bindings/BestFriendResponse';
import ItemDetailsModal from './ItemDetailsModal';
import {
    SortableHeader,
    StaticHeader,
    StatsTable,
    StatsTbodyRow,
    StatsTheadRow,
    useSortState,
} from './StatsTable';

type SortKey = 'score' | 'char1' | 'char2' | 'together' | 'pctC1' | 'pctC2';

function normalizedScore(
    comicsTogether: number,
    app1: number,
    app2: number,
): number {
    const minApp = Math.min(app1, app2);
    if (minApp === 0) return 0;
    return comicsTogether / minApp;
}

function pct(together: number, appearances: number) {
    if (appearances === 0) return '—';
    return `${((together / appearances) * 100).toFixed(1)}%`;
}

export default function BestFriendScore() {
    const [data, setData] = useState<BestFriendResponse | null>(null);
    const [error, setError] = useState<string | null>(null);
    const [sort, handleSort] = useSortState<SortKey>('score', 'desc');
    const [selectedItemId, setSelectedItemId] = useState<number | null>(null);

    useEffect(() => {
        fetch('/api/v3/stats/best-friend-score')
            .then((r) => {
                if (!r.ok) throw new Error(`HTTP ${r.status}`);
                return r.json() as Promise<BestFriendResponse>;
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
            const scoreA = normalizedScore(
                a.comicsTogether,
                c1a?.appearances ?? 0,
                c2a?.appearances ?? 0,
            );
            const scoreB = normalizedScore(
                b.comicsTogether,
                c1b?.appearances ?? 0,
                c2b?.appearances ?? 0,
            );
            let diff: number;
            if (sort.key === 'char1') {
                diff = (c1a?.name ?? '').localeCompare(c1b?.name ?? '');
            } else if (sort.key === 'char2') {
                diff = (c2a?.name ?? '').localeCompare(c2b?.name ?? '');
            } else if (sort.key === 'together') {
                diff = a.comicsTogether - b.comicsTogether;
            } else if (sort.key === 'pctC1') {
                const pA = c1a ? a.comicsTogether / c1a.appearances : 0;
                const pB = c1b ? b.comicsTogether / c1b.appearances : 0;
                diff = pA - pB;
            } else if (sort.key === 'pctC2') {
                const pA = c2a ? a.comicsTogether / c2a.appearances : 0;
                const pB = c2b ? b.comicsTogether / c2b.appearances : 0;
                diff = pA - pB;
            } else {
                diff = scoreA - scoreB;
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
                    Best Friend Score
                </h2>
                <p className="text-sm text-gray-500 mb-4">
                    Character pairs ranked by normalized co-appearance: comics
                    together divided by the smaller character's total
                    appearances. The closer to 100% the score is, the more often
                    the less common character appears with the other. Requires
                    at least 5 shared comics.
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
                                Together
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
                            <SortableHeader
                                sortKey="score"
                                sort={sort}
                                onSort={handleSort}
                            >
                                Score
                            </SortableHeader>
                        </StatsTheadRow>
                    </thead>
                    <tbody>
                        {sortedPairs.map((pair, i) => {
                            const c1 = data.characters[pair.character1Id];
                            const c2 = data.characters[pair.character2Id];
                            const score = normalizedScore(
                                pair.comicsTogether,
                                c1?.appearances ?? 0,
                                c2?.appearances ?? 0,
                            );
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
                                    <td className="py-2 pr-4 text-right text-gray-500">
                                        {c2
                                            ? pct(
                                                  pair.comicsTogether,
                                                  c2.appearances,
                                              )
                                            : '—'}
                                    </td>
                                    <td className="py-2 text-right font-medium text-indigo-700">
                                        {(score * 100).toFixed(1)}%
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
