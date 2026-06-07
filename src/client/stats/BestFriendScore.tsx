import { useEffect, useMemo, useState } from 'react';

import type { BestFriendResponse } from '../../../bindings/BestFriendResponse';

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
            const ca = data.characters[a.character1Id];
            const cb1 = data.characters[b.character1Id];
            const ca2 = data.characters[a.character2Id];
            const cb2 = data.characters[b.character2Id];
            const scoreA = normalizedScore(
                a.comicsTogether,
                ca?.appearances ?? 0,
                ca2?.appearances ?? 0,
            );
            const scoreB = normalizedScore(
                b.comicsTogether,
                cb1?.appearances ?? 0,
                cb2?.appearances ?? 0,
            );
            return scoreB - scoreA;
        });
    }, [data]);

    if (error) {
        return <p className="text-red-600">Failed to load data: {error}</p>;
    }

    if (!sortedPairs || !data) {
        return <p className="text-gray-500">Loading…</p>;
    }

    return (
        <div>
            <h2 className="text-xl font-semibold text-gray-800 mb-1">
                Best Friend Score
            </h2>
            <p className="text-sm text-gray-500 mb-4">
                Character pairs ranked by normalized co-appearance: comics
                together divided by the smaller character's total appearances. A
                score near 100% means the less common character almost always
                appears with the other. Requires at least 5 shared comics.
            </p>
            <div className="overflow-x-auto">
                <table className="min-w-full text-sm">
                    <thead>
                        <tr className="border-b border-gray-200 text-left text-gray-600">
                            <th className="py-2 pr-4 font-medium w-12">#</th>
                            <th className="py-2 pr-4 font-medium">
                                Character 1
                            </th>
                            <th className="py-2 pr-4 font-medium">
                                Character 2
                            </th>
                            <th className="py-2 pr-4 font-medium text-right">
                                Together
                            </th>
                            <th className="py-2 pr-4 font-medium text-right">
                                % of C1
                            </th>
                            <th className="py-2 pr-4 font-medium text-right">
                                % of C2
                            </th>
                            <th className="py-2 font-medium text-right">
                                Score
                            </th>
                        </tr>
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
                                <tr
                                    key={`${pair.character1Id}-${pair.character2Id}`}
                                    className="border-b border-gray-100 hover:bg-gray-50"
                                >
                                    <td className="py-2 pr-4 text-gray-400">
                                        {i + 1}
                                    </td>
                                    <td className="py-2 pr-4 font-medium text-gray-900">
                                        {c1?.name}
                                    </td>
                                    <td className="py-2 pr-4 font-medium text-gray-900">
                                        {c2?.name}
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
                                </tr>
                            );
                        })}
                    </tbody>
                </table>
            </div>
        </div>
    );
}
