import { useEffect, useMemo, useState } from 'react';

import type { TrendingItem } from '../../../bindings/TrendingItem';
import ItemDetailsModal from './ItemDetailsModal';

type SortKey = 'recent' | 'total' | 'ratio' | 'name';
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

function trendRatio(item: TrendingItem): number {
    const avgPerYear =
        item.careerYears > 0 ? item.totalAppearances / item.careerYears : 0;
    if (avgPerYear === 0) return 0;
    return item.recentAppearances / avgPerYear;
}

export default function TrendingCharacters() {
    const [data, setData] = useState<TrendingItem[] | null>(null);
    const [error, setError] = useState<string | null>(null);
    const [sort, setSort] = useState<SortState>({ key: 'ratio', dir: 'desc' });
    const [selectedItemId, setSelectedItemId] = useState<number | null>(null);

    useEffect(() => {
        fetch('/api/v3/stats/trending-characters')
            .then((r) => {
                if (!r.ok) throw new Error(`HTTP ${r.status}`);
                return r.json() as Promise<TrendingItem[]>;
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
                sort.key === 'recent'
                    ? a.recentAppearances - b.recentAppearances
                    : sort.key === 'total'
                      ? a.totalAppearances - b.totalAppearances
                      : sort.key === 'ratio'
                        ? trendRatio(a) - trendRatio(b)
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
                    Trending Characters
                </h2>
                <p className="text-sm text-gray-500 mb-4">
                    Characters whose appearances in the last 12 months are high
                    relative to their historical average per year. A ratio above
                    1.0× means they are appearing more than usual recently. Only
                    characters with at least 5 career appearances are shown.
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
                                    label="Recent (12 mo)"
                                    sortKey="recent"
                                    current={sort}
                                    onSort={handleSort}
                                />
                                <SortHeader
                                    label="Career avg/yr"
                                    sortKey="total"
                                    current={sort}
                                    onSort={handleSort}
                                />
                                <SortHeader
                                    label="Trend ratio"
                                    sortKey="ratio"
                                    current={sort}
                                    onSort={handleSort}
                                />
                            </tr>
                        </thead>
                        <tbody>
                            {sorted.map((row, i) => {
                                const avgPerYear =
                                    row.careerYears > 0
                                        ? row.totalAppearances / row.careerYears
                                        : 0;
                                const ratio = trendRatio(row);
                                return (
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
                                        <td className="py-2 pr-4 text-right font-medium text-indigo-700">
                                            {row.recentAppearances.toLocaleString()}
                                        </td>
                                        <td className="py-2 pr-4 text-right text-gray-500">
                                            {avgPerYear.toFixed(1)}
                                        </td>
                                        <td
                                            className={`py-2 text-right font-medium ${ratio >= 1.5 ? 'text-green-600' : ratio >= 1 ? 'text-indigo-700' : 'text-gray-500'}`}
                                        >
                                            {ratio.toFixed(2)}×
                                        </td>
                                    </tr>
                                );
                            })}
                        </tbody>
                    </table>
                </div>
            </div>
        </>
    );
}
