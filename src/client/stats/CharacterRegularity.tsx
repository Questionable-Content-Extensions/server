import { useEffect, useMemo, useState } from 'react';

import type { CharacterRegularity } from '../../../bindings/CharacterRegularity';

type SortKey = 'stddev' | 'avg' | 'name' | 'appearances';
type SortDir = 'asc' | 'desc';

interface SortState {
    key: SortKey;
    dir: SortDir;
}

function formatDays(days: number) {
    if (days >= 30) {
        return `${(days / 30).toFixed(1)}mo`;
    }
    return `${days.toFixed(1)}d`;
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

export default function CharacterRegularityPage() {
    const [data, setData] = useState<CharacterRegularity[] | null>(null);
    const [error, setError] = useState<string | null>(null);
    const [sort, setSort] = useState<SortState>({
        key: 'stddev',
        dir: 'asc',
    });

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
        <div>
            <h2 className="text-xl font-semibold text-gray-800 mb-1">
                Character Regularity
            </h2>
            <p className="text-sm text-gray-500 mb-4">
                For characters with at least 10 dated appearances, the standard
                deviation of days between consecutive appearances. Low std dev
                means very consistent; high means bursty or arc-driven. Click
                any column header to sort. "Dated appearances" counts only
                comics with a known publish date.
            </p>
            <div className="overflow-x-auto">
                <table className="min-w-full text-sm">
                    <thead>
                        <tr className="border-b border-gray-200 text-left text-gray-600">
                            <th className="py-2 pr-4 font-medium w-12">#</th>
                            <SortHeader
                                label="Name"
                                sortKey="name"
                                current={sort}
                                onSort={handleSort}
                                align="left"
                            />
                            <SortHeader
                                label="Dated appearances"
                                sortKey="appearances"
                                current={sort}
                                onSort={handleSort}
                            />
                            <SortHeader
                                label="Avg gap"
                                sortKey="avg"
                                current={sort}
                                onSort={handleSort}
                            />
                            <SortHeader
                                label="Std dev"
                                sortKey="stddev"
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
                                <td className="py-2 pr-4 font-medium text-gray-900">
                                    {row.name}
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
                            </tr>
                        ))}
                    </tbody>
                </table>
            </div>
        </div>
    );
}
