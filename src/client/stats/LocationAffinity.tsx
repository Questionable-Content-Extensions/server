import { useEffect, useState } from 'react';

import type { LocationAffinity } from '../../../bindings/LocationAffinity';

export default function LocationAffinityPage() {
    const [data, setData] = useState<LocationAffinity[] | null>(null);
    const [error, setError] = useState<string | null>(null);
    const [expanded, setExpanded] = useState<Set<number>>(new Set());

    useEffect(() => {
        fetch('/api/v3/stats/location-affinity')
            .then((r) => {
                if (!r.ok) throw new Error(`HTTP ${r.status}`);
                return r.json() as Promise<LocationAffinity[]>;
            })
            .then(setData)
            .catch((e: unknown) =>
                setError(e instanceof Error ? e.message : String(e)),
            );
    }, []);

    function toggle(locationId: number) {
        setExpanded((prev) => {
            const next = new Set(prev);
            if (next.has(locationId)) {
                next.delete(locationId);
            } else {
                next.add(locationId);
            }
            return next;
        });
    }

    function expandAll() {
        if (!data) return;
        setExpanded(new Set(data.map((loc) => loc.locationId)));
    }

    function collapseAll() {
        setExpanded(new Set());
    }

    if (error) {
        return <p className="text-red-600">Failed to load data: {error}</p>;
    }

    if (!data) {
        return <p className="text-gray-500">Loading…</p>;
    }

    return (
        <div>
            <h2 className="text-xl font-semibold text-gray-800 mb-1">
                Location–Character Affinity
            </h2>
            <div className="flex items-center gap-4 mb-4">
                <p className="text-sm text-gray-500">
                    For each location, the top 5 characters who appear there
                    most. Click a location to expand.
                </p>
                <div className="ml-auto flex gap-2 shrink-0">
                    <button
                        type="button"
                        className="text-xs text-indigo-600 hover:underline"
                        onClick={expandAll}
                    >
                        Expand all
                    </button>
                    <button
                        type="button"
                        className="text-xs text-indigo-600 hover:underline"
                        onClick={collapseAll}
                    >
                        Collapse all
                    </button>
                </div>
            </div>
            <div className="divide-y divide-gray-100">
                {data.map((loc) => {
                    const isOpen = expanded.has(loc.locationId);
                    return (
                        <div key={loc.locationId}>
                            <button
                                type="button"
                                className="w-full text-left py-3 px-1 flex items-center justify-between hover:bg-gray-50 transition-colors"
                                onClick={() => {
                                    toggle(loc.locationId);
                                }}
                            >
                                <span className="font-medium text-gray-900">
                                    {loc.locationName}
                                </span>
                                <span className="text-gray-400 text-sm ml-2">
                                    {isOpen ? '▲' : '▼'}
                                </span>
                            </button>
                            {isOpen && (
                                <div className="pb-3 pl-4">
                                    <table className="text-sm min-w-full">
                                        <thead>
                                            <tr className="text-left text-gray-500">
                                                <th className="pb-1 pr-4 font-medium">
                                                    Character
                                                </th>
                                                <th className="pb-1 font-medium text-right">
                                                    Comics together
                                                </th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {loc.topCharacters.map((ch) => (
                                                <tr
                                                    key={ch.id}
                                                    className="border-t border-gray-100"
                                                >
                                                    <td className="py-1.5 pr-4 text-gray-800">
                                                        {ch.name}
                                                    </td>
                                                    <td className="py-1.5 text-right text-gray-600">
                                                        {ch.comicsTogether.toLocaleString()}
                                                    </td>
                                                </tr>
                                            ))}
                                        </tbody>
                                    </table>
                                </div>
                            )}
                        </div>
                    );
                })}
            </div>
        </div>
    );
}
