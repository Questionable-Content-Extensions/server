import { useEffect, useMemo, useState } from 'react';

import type { TrendingItem } from '../../../bindings/TrendingItem';
import ItemDetailsModal from './ItemDetailsModal';
import {
    SortableHeader,
    StaticHeader,
    StatsTable,
    StatsTbodyRow,
    StatsTheadRow,
    useSortState,
} from './StatsTable';

type SortKey = 'recent' | 'total' | 'ratio' | 'name';

function trendRatio(item: TrendingItem): number {
    const avgPerYear =
        item.careerYears > 0 ? item.totalAppearances / item.careerYears : 0;
    if (avgPerYear === 0) return 0;
    return item.recentAppearances / avgPerYear;
}

export default function TrendingLocations() {
    const [data, setData] = useState<TrendingItem[] | null>(null);
    const [error, setError] = useState<string | null>(null);
    const [sort, handleSort] = useSortState<SortKey>('ratio', 'desc');
    const [selectedItemId, setSelectedItemId] = useState<number | null>(null);

    useEffect(() => {
        fetch('/api/v3/stats/trending-locations')
            .then((r) => {
                if (!r.ok) throw new Error(`HTTP ${r.status}`);
                return r.json() as Promise<TrendingItem[]>;
            })
            .then(setData)
            .catch((e: unknown) =>
                setError(e instanceof Error ? e.message : String(e)),
            );
    }, []);

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
                    Trending Locations
                </h2>
                <p className="text-sm text-gray-500 mb-4">
                    Locations whose appearances in the last 12 months are high
                    relative to their historical average per year. A ratio above
                    1.0× means the location is appearing more than usual
                    recently. Only locations with at least 5 career appearances
                    are shown.
                </p>
                <StatsTable>
                    <thead>
                        <StatsTheadRow>
                            <StaticHeader className="w-12">#</StaticHeader>
                            <SortableHeader
                                sortKey="name"
                                sort={sort}
                                onSort={handleSort}
                                align="left"
                            >
                                Name
                            </SortableHeader>
                            <SortableHeader
                                sortKey="recent"
                                sort={sort}
                                onSort={handleSort}
                            >
                                Recent (12 mo)
                            </SortableHeader>
                            <SortableHeader
                                sortKey="total"
                                sort={sort}
                                onSort={handleSort}
                            >
                                Career avg/yr
                            </SortableHeader>
                            <SortableHeader
                                sortKey="ratio"
                                sort={sort}
                                onSort={handleSort}
                            >
                                Trend ratio
                            </SortableHeader>
                        </StatsTheadRow>
                    </thead>
                    <tbody>
                        {sorted.map((row, i) => {
                            const avgPerYear =
                                row.careerYears > 0
                                    ? row.totalAppearances / row.careerYears
                                    : 0;
                            const ratio = trendRatio(row);
                            return (
                                <StatsTbodyRow key={row.id}>
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
                                </StatsTbodyRow>
                            );
                        })}
                    </tbody>
                </StatsTable>
            </div>
        </>
    );
}
