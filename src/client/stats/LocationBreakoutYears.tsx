import { useEffect, useMemo, useState } from 'react';

import type { LocationBreakoutYear } from '../../../bindings/LocationBreakoutYear';
import ItemDetailsModal from './ItemDetailsModal';
import {
    SortableHeader,
    StaticHeader,
    StatsTable,
    StatsTbodyRow,
    StatsTheadRow,
    useSortState,
} from './StatsTable';

type SortKey = 'ratio' | 'count' | 'avg' | 'year' | 'name';

export default function LocationBreakoutYears() {
    const [data, setData] = useState<LocationBreakoutYear[] | null>(null);
    const [error, setError] = useState<string | null>(null);
    const [sort, handleSort] = useSortState<SortKey>('ratio', 'desc');
    const [selectedItemId, setSelectedItemId] = useState<number | null>(null);

    useEffect(() => {
        fetch('/api/v3/stats/location-breakout-years')
            .then((r) => {
                if (!r.ok) throw new Error(`HTTP ${r.status}`);
                return r.json() as Promise<LocationBreakoutYear[]>;
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
                    Location Breakout Years
                </h2>
                <p className="text-sm text-gray-500 mb-4">
                    Each location&apos;s best year (highest appearance count),
                    compared to their career average per year. The ratio shows
                    how exceptional that year was — a ratio of 3× means the
                    location appeared three times more than an average year.
                    When multiple years tie for the best count, all are listed.
                    Only locations with appearances in at least 2 years are
                    included.
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
                                sortKey="year"
                                sort={sort}
                                onSort={handleSort}
                            >
                                Breakout year
                            </SortableHeader>
                            <SortableHeader
                                sortKey="count"
                                sort={sort}
                                onSort={handleSort}
                            >
                                Count
                            </SortableHeader>
                            <SortableHeader
                                sortKey="avg"
                                sort={sort}
                                onSort={handleSort}
                            >
                                Avg/yr
                            </SortableHeader>
                            <SortableHeader
                                sortKey="ratio"
                                sort={sort}
                                onSort={handleSort}
                            >
                                Ratio
                            </SortableHeader>
                        </StatsTheadRow>
                    </thead>
                    <tbody>
                        {sorted.map((row, i) => (
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
                            </StatsTbodyRow>
                        ))}
                    </tbody>
                </StatsTable>
            </div>
        </>
    );
}
