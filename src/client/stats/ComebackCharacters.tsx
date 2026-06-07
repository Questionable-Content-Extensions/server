import { useEffect, useState } from 'react';

import type { ComebackCharacter } from '../../../bindings/ComebackCharacter';

function comicLink(comicId: number) {
    return `https://questionablecontent.net/view.php?comic=${comicId}`;
}

function formatGap(days: number) {
    if (days >= 365) {
        const years = (days / 365).toFixed(1);
        return `${years}y (${days.toLocaleString()}d)`;
    }
    return `${days.toLocaleString()} days`;
}

export default function ComebackCharacters() {
    const [data, setData] = useState<ComebackCharacter[] | null>(null);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        fetch('/api/v3/stats/comeback-characters')
            .then((r) => {
                if (!r.ok) throw new Error(`HTTP ${r.status}`);
                return r.json() as Promise<ComebackCharacter[]>;
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
                Comeback Characters
            </h2>
            <p className="text-sm text-gray-500 mb-4">
                Characters with the longest gap between two consecutive
                appearances — those who disappeared and came back. Top 50 by
                largest single gap (minimum 90 days).
            </p>
            <div className="overflow-x-auto">
                <table className="min-w-full text-sm">
                    <thead>
                        <tr className="border-b border-gray-200 text-left text-gray-600">
                            <th className="py-2 pr-4 font-medium w-12">#</th>
                            <th className="py-2 pr-4 font-medium">Name</th>
                            <th className="py-2 pr-4 font-medium text-right">
                                Last seen
                            </th>
                            <th className="py-2 pr-4 font-medium text-right">
                                Returned in
                            </th>
                            <th className="py-2 font-medium text-right">Gap</th>
                        </tr>
                    </thead>
                    <tbody>
                        {data.map((row, i) => (
                            <tr
                                key={row.id}
                                className="border-b border-gray-100 hover:bg-gray-50"
                            >
                                <td className="py-2 pr-4 text-gray-400">
                                    {i + 1}
                                </td>
                                <td className="py-2 pr-4 font-medium text-gray-900">
                                    {row.name}
                                </td>
                                <td className="py-2 pr-4 text-right">
                                    <a
                                        href={comicLink(row.lastComic)}
                                        className="text-blue-600 hover:underline"
                                        target="_blank"
                                        rel="noreferrer"
                                    >
                                        #{row.lastComic}
                                    </a>
                                </td>
                                <td className="py-2 pr-4 text-right">
                                    <a
                                        href={comicLink(row.returnComic)}
                                        className="text-blue-600 hover:underline"
                                        target="_blank"
                                        rel="noreferrer"
                                    >
                                        #{row.returnComic}
                                    </a>
                                </td>
                                <td className="py-2 text-right text-gray-700">
                                    {formatGap(row.gapDays)}
                                </td>
                            </tr>
                        ))}
                    </tbody>
                </table>
            </div>
        </div>
    );
}
