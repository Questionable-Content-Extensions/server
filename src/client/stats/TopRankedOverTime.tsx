import { useEffect, useMemo, useState } from 'react';

import type { TopRankedOverTimeResponse } from 'bindings/TopRankedOverTimeResponse';
import { getStatsTopRankedOverTime } from 'bindings/api/GetStatsTopRankedOverTime';

interface StintRow {
    rank: number;
    itemId: number;
    name: string;
    color: string;
    fromComic: number;
    toComicInclusive: number | null;
    comicCount: number | null;
    appearancesAtTakeover: number;
}

function colorToStyle(color: string): React.CSSProperties {
    return { color: `#${color}` };
}

function comicRange(from: number, toInclusive: number | null): string {
    if (toInclusive === null) return `#${from} – present`;
    return `#${from} – #${toInclusive}`;
}

export default function TopRankedOverTime() {
    const [response, setResponse] = useState<TopRankedOverTimeResponse | null>(
        null,
    );
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        getStatsTopRankedOverTime()
            .then(setResponse)
            .catch((e: unknown) =>
                setError(e instanceof Error ? e.message : String(e)),
            );
    }, []);

    const rows = useMemo<StintRow[] | null>(() => {
        if (!response) return null;
        return response.stints.map((stint, i) => {
            const meta = response.characters[stint.itemId];
            const toExcl = stint.toComicExclusive;
            const toInclusive = toExcl !== null ? toExcl - 1 : null;
            const comicCount =
                toInclusive !== null ? toInclusive - stint.fromComic + 1 : null;
            return {
                rank: i + 1,
                itemId: stint.itemId,
                name: meta?.name ?? `Character #${stint.itemId}`,
                color: meta?.color ?? '888888',
                fromComic: stint.fromComic,
                toComicInclusive: toInclusive,
                comicCount,
                appearancesAtTakeover: stint.appearancesAtTakeover,
            };
        });
    }, [response]);

    if (error)
        return <p className="text-red-600">Failed to load data: {error}</p>;
    if (!rows) return <p className="text-gray-500">Loading…</p>;

    return (
        <div>
            <h2 className="text-xl font-semibold text-gray-800 mb-1">
                Top Rank Over Time
            </h2>
            <p className="text-sm text-gray-500 mb-4">
                Tracks which character held the top cumulative appearance rank
                at each point in the comic. A character takes the lead when
                their running total exceeds all others; ties are broken in
                favour of the incumbent (the current leader keeps the top spot
                until strictly overtaken).
            </p>
            <table className="w-full text-sm border-collapse">
                <thead>
                    <tr className="border-b border-gray-200 text-left text-xs font-semibold uppercase tracking-wide text-gray-500">
                        <th className="pb-2 pr-4 w-10">#</th>
                        <th className="pb-2 pr-4">Character</th>
                        <th className="pb-2 pr-4">Comic range</th>
                        <th className="pb-2 pr-4 text-right">Comics held</th>
                        <th className="pb-2 text-right">
                            Appearances at takeover
                        </th>
                    </tr>
                </thead>
                <tbody>
                    {rows.map((row) => (
                        <tr
                            key={row.rank}
                            className="border-b border-gray-100 last:border-b-0 hover:bg-gray-50"
                        >
                            <td className="py-2 pr-4 text-gray-400">
                                {row.rank}
                            </td>
                            <td
                                className="py-2 pr-4 font-semibold"
                                style={colorToStyle(row.color)}
                            >
                                {row.name}
                            </td>
                            <td className="py-2 pr-4 text-gray-700 font-mono text-xs">
                                {comicRange(
                                    row.fromComic,
                                    row.toComicInclusive,
                                )}
                            </td>
                            <td className="py-2 pr-4 text-right text-gray-600">
                                {row.comicCount !== null
                                    ? row.comicCount.toLocaleString()
                                    : '—'}
                            </td>
                            <td className="py-2 text-right font-medium text-indigo-700">
                                {row.appearancesAtTakeover.toLocaleString()}
                            </td>
                        </tr>
                    ))}
                </tbody>
            </table>
        </div>
    );
}
