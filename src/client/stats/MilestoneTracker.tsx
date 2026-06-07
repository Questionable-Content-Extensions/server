import { useEffect, useState } from 'react';

import type { MilestoneComic } from '../../../bindings/MilestoneComic';

export default function MilestoneTracker() {
    const [data, setData] = useState<MilestoneComic[] | null>(null);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        fetch('/api/v3/stats/milestones')
            .then((r) => {
                if (!r.ok) throw new Error(`HTTP ${r.status}`);
                return r.json() as Promise<MilestoneComic[]>;
            })
            .then(setData)
            .catch((e: unknown) =>
                setError(e instanceof Error ? e.message : String(e)),
            );
    }, []);

    if (error)
        return <p className="text-red-600">Failed to load data: {error}</p>;
    if (!data) return <p className="text-gray-500">Loading…</p>;

    return (
        <div>
            <h2 className="text-xl font-semibold text-gray-800 mb-1">
                Milestone Tracker
            </h2>
            <p className="text-sm text-gray-500 mb-4">
                Round-number comics that serve as milestones in the archive: #1
                (the beginning), every 100th up to #1000, and every 500th beyond
                that.
            </p>
            <div className="overflow-x-auto">
                <table className="min-w-full text-sm">
                    <thead>
                        <tr className="border-b border-gray-200 text-left text-gray-600">
                            <th className="py-2 pr-4 font-medium">#</th>
                            <th className="py-2 pr-4 font-medium">Title</th>
                            <th className="py-2 pr-4 font-medium">Date</th>
                            <th className="py-2 font-medium">Flags</th>
                        </tr>
                    </thead>
                    <tbody>
                        {data.map((row) => (
                            <tr
                                key={row.comicId}
                                className="border-b border-gray-100 hover:bg-gray-50"
                            >
                                <td className="py-2 pr-4 font-medium text-indigo-700">
                                    #{row.comicId}
                                </td>
                                <td className="py-2 pr-4 text-gray-900">
                                    {row.title || (
                                        <span className="text-gray-400 italic">
                                            Untitled
                                        </span>
                                    )}
                                </td>
                                <td className="py-2 pr-4 text-gray-500">
                                    {row.pubDate ?? '—'}
                                </td>
                                <td className="py-2 text-gray-500">
                                    {row.isGuestComic && (
                                        <span className="inline-block mr-1 px-1.5 py-0.5 rounded bg-yellow-100 text-yellow-700 text-xs">
                                            Guest
                                        </span>
                                    )}
                                    {row.isNonCanon && (
                                        <span className="inline-block px-1.5 py-0.5 rounded bg-gray-100 text-gray-500 text-xs">
                                            Non-canon
                                        </span>
                                    )}
                                </td>
                            </tr>
                        ))}
                    </tbody>
                </table>
            </div>
        </div>
    );
}
