import { useEffect, useMemo, useState } from 'react';

import type { CharacterRegularity } from '../../../bindings/CharacterRegularity';
import ItemDetailsModal from './ItemDetailsModal';
import {
    SortableHeader,
    StaticHeader,
    StatsTable,
    StatsTbodyRow,
    StatsTheadRow,
    useSortState,
} from './StatsTable';

type SortKey = 'stddev' | 'avg' | 'name' | 'appearances';

function formatDays(days: number) {
    if (days >= 30) {
        return `${(days / 30).toFixed(1)}mo`;
    }
    return `${days.toFixed(1)}d`;
}

export default function CharacterRegularityPage() {
    const [data, setData] = useState<CharacterRegularity[] | null>(null);
    const [error, setError] = useState<string | null>(null);
    const [sort, handleSort] = useSortState<SortKey>('stddev', 'asc');
    const [selectedItemId, setSelectedItemId] = useState<number | null>(null);

    useEffect(() => {
        fetch('/api/v3/stats/character-regularity')
            .then((r) => {
                if (!r.ok) throw new Error(`HTTP ${r.status}`);
                return r.json() as Promise<CharacterRegularity[]>;
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
                sort.key === 'stddev'
                    ? a.stddevGapDays - b.stddevGapDays
                    : sort.key === 'avg'
                      ? a.avgGapDays - b.avgGapDays
                      : sort.key === 'appearances'
                        ? a.appearances - b.appearances
                        : a.name.localeCompare(b.name);
            return sort.dir === 'asc' ? diff : -diff;
        });
        return copy;
    }, [data, sort]);

    if (error) {
        return <p className="text-red-600">Failed to load data: {error}</p>;
    }

    if (!sorted) {
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
                    Character Regularity
                </h2>
                <p className="text-sm text-gray-500 mb-4">
                    For characters with at least 10 dated appearances, the
                    standard deviation of days between consecutive appearances.
                    Low std dev means very consistent; high means bursty or
                    arc-driven. Click any column header to sort. "Dated
                    appearances" counts only comics with a known publish date.
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
                                sortKey="appearances"
                                sort={sort}
                                onSort={handleSort}
                            >
                                Dated appearances
                            </SortableHeader>
                            <SortableHeader
                                sortKey="avg"
                                sort={sort}
                                onSort={handleSort}
                            >
                                Avg gap
                            </SortableHeader>
                            <SortableHeader
                                sortKey="stddev"
                                sort={sort}
                                onSort={handleSort}
                            >
                                Std dev
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
                                <td className="py-2 pr-4 text-right text-gray-700">
                                    {row.appearances.toLocaleString()}
                                </td>
                                <td className="py-2 pr-4 text-right text-gray-700">
                                    {formatDays(row.avgGapDays)}
                                </td>
                                <td className="py-2 text-right font-medium text-gray-800">
                                    {formatDays(row.stddevGapDays)}
                                </td>
                            </StatsTbodyRow>
                        ))}
                    </tbody>
                </StatsTable>
            </div>
        </>
    );
}
