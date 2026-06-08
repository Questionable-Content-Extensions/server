import { useEffect, useState } from 'react';

import type { DebutYear } from '../../../bindings/DebutYear';
import ItemDetailsModal from './ItemDetailsModal';

export default function DebutYears() {
    const [data, setData] = useState<DebutYear[] | null>(null);
    const [error, setError] = useState<string | null>(null);
    const [expanded, setExpanded] = useState<Set<number>>(new Set());
    const [selectedItemId, setSelectedItemId] = useState<number | null>(null);

    useEffect(() => {
        fetch('/api/v3/stats/debut-clusters')
            .then((r) => {
                if (!r.ok) throw new Error(`HTTP ${r.status}`);
                return r.json() as Promise<DebutYear[]>;
            })
            .then(setData)
            .catch((e: unknown) =>
                setError(e instanceof Error ? e.message : String(e)),
            );
    }, []);

    function toggle(year: number) {
        setExpanded((prev) => {
            const next = new Set(prev);
            if (next.has(year)) {
                next.delete(year);
            } else {
                next.add(year);
            }
            return next;
        });
    }

    function expandAll() {
        if (data) setExpanded(new Set(data.map((d) => d.year)));
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
                    Character Debut Years
                </h2>
                <div className="flex items-center gap-4 mb-4">
                    <p className="text-sm text-gray-500">
                        Every character grouped by the year of their first
                        appearance. Click a year to expand.
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
                    {data.map((entry) => {
                        const isOpen = expanded.has(entry.year);
                        return (
                            <div key={entry.year}>
                                <button
                                    type="button"
                                    className="w-full text-left py-3 px-1 flex items-center justify-between hover:bg-gray-50 transition-colors"
                                    onClick={() => {
                                        toggle(entry.year);
                                    }}
                                >
                                    <span className="font-medium text-gray-900">
                                        {entry.year}
                                    </span>
                                    <span className="text-gray-400 text-sm ml-2 flex items-center gap-2">
                                        <span className="text-gray-500">
                                            {entry.characters.length} character
                                            {entry.characters.length !== 1
                                                ? 's'
                                                : ''}
                                        </span>
                                        {isOpen ? '▲' : '▼'}
                                    </span>
                                </button>
                                {isOpen && (
                                    <div className="pb-3 pl-4">
                                        <div className="flex flex-wrap gap-x-4 gap-y-1">
                                            {entry.characters.map((ch) => (
                                                <button
                                                    key={ch.id}
                                                    type="button"
                                                    onClick={() => {
                                                        setSelectedItemId(
                                                            ch.id,
                                                        );
                                                    }}
                                                    className="text-sm text-gray-700 hover:text-blue-600 hover:underline text-left"
                                                >
                                                    {ch.name}
                                                </button>
                                            ))}
                                        </div>
                                    </div>
                                )}
                            </div>
                        );
                    })}
                </div>
            </div>
        </>
    );
}
