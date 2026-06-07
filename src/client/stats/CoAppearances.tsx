import { useEffect, useState } from 'react';

import type { CoAppearancesResponse } from '../../../bindings/CoAppearancesResponse';

function pct(together: number, appearances: number) {
    if (appearances === 0) return '—';
    return `${((together / appearances) * 100).toFixed(1)}%`;
}

export default function CoAppearances() {
    const [data, setData] = useState<CoAppearancesResponse | null>(null);
    const [error, setError] = useState<string | null>(null);

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

    if (error) {
        return <p className="text-red-600">Failed to load data: {error}</p>;
    }

    if (!data) {
        return <p className="text-gray-500">Loading…</p>;
    }

    return (
        <div>
            <h2 className="text-xl font-semibold text-gray-800 mb-1">
                Who Appears Together
            </h2>
            <p className="text-sm text-gray-500 mb-4">
                The top 100 character pairs by shared comic appearances. The %
                columns show what share of each character's total appearances
                include the other.
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
                                Comics together
                            </th>
                            <th className="py-2 pr-4 font-medium text-right">
                                % of C1
                            </th>
                            <th className="py-2 font-medium text-right">
                                % of C2
                            </th>
                        </tr>
                    </thead>
                    <tbody>
                        {data.pairs.map((pair, i) => {
                            const c1 = data.characters[pair.character1Id];
                            const c2 = data.characters[pair.character2Id];
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
                                    <td className="py-2 text-right text-gray-500">
                                        {c2
                                            ? pct(
                                                  pair.comicsTogether,
                                                  c2.appearances,
                                              )
                                            : '—'}
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
