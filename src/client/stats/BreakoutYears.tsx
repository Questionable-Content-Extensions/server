import { useEffect, useMemo, useState } from 'react';

import type { BreakoutYear } from '../../../bindings/BreakoutYear';
import ItemDetailsModal from './ItemDetailsModal';

type SortKey = 'ratio' | 'count' | 'avg' | 'year' | 'name';
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

export default function BreakoutYears() {
    const [data, setData] = useState<BreakoutYear[] | null>(null);
    const [error, setError] = useState<string | null>(null);
    const [sort, setSort] = useState<SortState>({ key: 'ratio', dir: 'desc' });
    const [selectedItemId, setSelectedItemId] = useState<number | null>(null);

    useEffect(() => {
        fetch('/api/v3/stats/breakout-years')
            .then((r) => {
                if (!r.ok) throw new Error(`HTTP ${r.status}`);
                return r.json() as Promise<BreakoutYear[]>;
            })
            .then(setData)
            .catch((e: unknown) =>
                setError(e instanceof Error ? e.message : String(e)),
            );
    }, []);

    function handleSort(key: SortKey) {
        setSort((prev) => ({
            key,
            dir: prev.key === key && prev.dir === 'desc' ? 'asc' : 'desc',
        }));
    }

    const sorted = useMemo(() => {
        if (!data) return null;
        const copy = [...data];
        copy.sort((a, b) => {
            const diff =
                sort.key === 'ratio'
                    ? a.ratio - b.ratio
                    : sort.key === 'count'
                      ? a.breakoutCount - b.breakoutCount
                      : sort.key === 'avg'
                        ? a.avgPerYear - b.avgPerYear
                        : sort.key === 'year'
                          ? (a.breakoutYears[0] ?? 0) -
                            (b.breakoutYears[0] ?? 0)
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
                    Breakout Years
                </h2>
                <p className="text-sm text-gray-500 mb-4">
                    Each character&apos;s best year (highest appearance count),
                    compared to their career average per year. The ratio shows
                    how exceptional that year was — a ratio of 3× means they
                    appeared three times more than usual. When multiple years
                    tie for the best count, all are listed. Only characters with
                    appearances in at least 2 years are included.
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
                                    label="Breakout year"
                                    sortKey="year"
                                    current={sort}
                                    onSort={handleSort}
                                />
                                <SortHeader
                                    label="Count"
                                    sortKey="count"
                                    current={sort}
                                    onSort={handleSort}
                                />
                                <SortHeader
                                    label="Avg/yr"
                                    sortKey="avg"
                                    current={sort}
                                    onSort={handleSort}
                                />
                                <SortHeader
                                    label="Ratio"
                                    sortKey="ratio"
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
                                        {row.breakoutYears.join(', ')}
                                    </td>
                                    <td className="py-2 pr-4 text-right font-medium text-indigo-700">
                                        {row.breakoutCount.toLocaleString()}
                                    </td>
                                    <td className="py-2 pr-4 text-right text-gray-500">
                                        {row.avgPerYear.toFixed(1)}
                                    </td>
                                    <td
                                        className={`py-2 text-right font-medium ${row.ratio >= 3 ? 'text-green-600' : row.ratio >= 2 ? 'text-indigo-700' : 'text-gray-500'}`}
                                    >
                                        {row.ratio.toFixed(2)}×
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
