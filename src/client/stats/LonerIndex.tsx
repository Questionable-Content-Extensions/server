import { useEffect, useMemo, useState } from 'react';

import type { LonerEntry } from '../../../bindings/LonerEntry';
import ItemDetailsModal from './ItemDetailsModal';

type SortKey = 'avg' | 'appearances' | 'name';
type SortDir = 'asc' | 'desc';

interface SortState {
    key: SortKey;
    dir: SortDir;
}

function SortHeader({
    label,
    sortKey,
    current,
    onSort,
    align = 'right',
}: {
    label: string;
    sortKey: SortKey;
    current: SortState;
    onSort: (key: SortKey) => void;
    align?: 'left' | 'right';
}) {
    const isActive = current.key === sortKey;
    const arrow = isActive ? (current.dir === 'asc' ? ' ↑' : ' ↓') : '';
    return (
        <th
            className={`py-2 pr-4 font-medium cursor-pointer select-none hover:text-gray-900 ${align === 'right' ? 'text-right' : 'text-left'} ${isActive ? 'text-gray-900' : ''}`}
            onClick={() => {
                onSort(sortKey);
            }}
        >
            {label}
            {arrow}
        </th>
    );
}

export default function LonerIndex() {
    const [data, setData] = useState<LonerEntry[] | null>(null);
    const [error, setError] = useState<string | null>(null);
    const [sort, setSort] = useState<SortState>({ key: 'avg', dir: 'asc' });
    const [selectedItemId, setSelectedItemId] = useState<number | null>(null);

    useEffect(() => {
        fetch('/api/v3/stats/loner-index')
            .then((r) => {
                if (!r.ok) throw new Error(`HTTP ${r.status}`);
                return r.json() as Promise<LonerEntry[]>;
            })
            .then(setData)
            .catch((e: unknown) =>
                setError(e instanceof Error ? e.message : String(e)),
            );
    }, []);

    function handleSort(key: SortKey) {
        setSort((prev) => ({
            key,
            dir: prev.key === key && prev.dir === 'asc' ? 'desc' : 'asc',
        }));
    }

    const sorted = useMemo(() => {
        if (!data) return null;
        const copy = [...data];
        copy.sort((a, b) => {
            const diff =
                sort.key === 'avg'
                    ? a.avgCoCast - b.avgCoCast
                    : sort.key === 'appearances'
                      ? a.appearances - b.appearances
                      : a.name.localeCompare(b.name);
            return sort.dir === 'asc' ? diff : -diff;
        });
        return copy;
    }, [data, sort]);

    if (error)
        return <p className="text-red-600">Failed to load data: {error}</p>;
    if (!sorted) return <p className="text-gray-500">Loading…</p>;

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
                    Loner Index
                </h2>
                <p className="text-sm text-gray-500 mb-4">
                    Characters sorted by how few co-stars they typically appear
                    with. The average co-cast size measures how many other cast
                    members share a comic with this character on average
                    (excluding themselves). A low value means the character
                    tends to appear alone or in very small scenes. Only
                    characters with at least 10 appearances are shown.
                </p>
                <div className="overflow-x-auto">
                    <table className="min-w-full text-sm">
                        <thead>
                            <tr className="border-b border-gray-200 text-left text-gray-600">
                                <th className="py-2 pr-4 font-medium w-12">
                                    #
                                </th>
                                <SortHeader
                                    label="Name"
                                    sortKey="name"
                                    current={sort}
                                    onSort={handleSort}
                                    align="left"
                                />
                                <SortHeader
                                    label="Appearances"
                                    sortKey="appearances"
                                    current={sort}
                                    onSort={handleSort}
                                />
                                <SortHeader
                                    label="Avg co-cast size"
                                    sortKey="avg"
                                    current={sort}
                                    onSort={handleSort}
                                />
                            </tr>
                        </thead>
                        <tbody>
                            {sorted.map((row, i) => (
                                <tr
                                    key={row.id}
                                    className="border-b border-gray-100 hover:bg-gray-50"
                                >
                                    <td className="py-2 pr-4 text-gray-400">
                                        {i + 1}
                                    </td>
                                    <td className="py-2 pr-4">
                                        <button
                                            type="button"
                                            onClick={() => {
                                                setSelectedItemId(row.id);
                                            }}
                                            className="font-medium text-gray-900 hover:text-blue-600 hover:underline text-left"
                                        >
                                            {row.name}
                                        </button>
                                    </td>
                                    <td className="py-2 pr-4 text-right text-gray-500">
                                        {row.appearances.toLocaleString()}
                                    </td>
                                    <td
                                        className={`py-2 text-right font-medium ${row.avgCoCast < 0.5 ? 'text-blue-600' : row.avgCoCast < 1.5 ? 'text-indigo-700' : 'text-gray-600'}`}
                                    >
                                        {row.avgCoCast.toFixed(2)}
                                    </td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                </div>
            </div>
        </>
    );
}
