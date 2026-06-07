import { useEffect, useMemo, useState } from 'react';

import type { PublicationStreak } from '../../../bindings/PublicationStreak';

type SortKey = 'days' | 'calendar' | 'start';
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

export default function PublicationStreaks() {
    const [data, setData] = useState<PublicationStreak[] | null>(null);
    const [error, setError] = useState<string | null>(null);
    const [sort, setSort] = useState<SortState>({ key: 'days', dir: 'desc' });

    useEffect(() => {
        fetch('/api/v3/stats/publication-streaks')
            .then((r) => {
                if (!r.ok) throw new Error(`HTTP ${r.status}`);
                return r.json() as Promise<PublicationStreak[]>;
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
                sort.key === 'days'
                    ? a.daysWithComics - b.daysWithComics
                    : sort.key === 'calendar'
                      ? a.calendarDays - b.calendarDays
                      : a.streakStart.localeCompare(b.streakStart);
            return sort.dir === 'asc' ? diff : -diff;
        });
        return copy;
    }, [data, sort]);

    if (error)
        return <p className="text-red-600">Failed to load data: {error}</p>;
    if (!sorted) return <p className="text-gray-500">Loading…</p>;

    return (
        <div>
            <h2 className="text-xl font-semibold text-gray-800 mb-1">
                Publication Streaks
            </h2>
            <p className="text-sm text-gray-500 mb-4">
                The longest consecutive weekday runs with at least one comic
                published. A streak is broken when a weekday (Monday–Friday)
                passes with no new comic — weekend gaps do not break a streak.
                Calendar days is the total span from start to end including
                weekends; days with comics is the actual count of publishing
                days. Top 20 streaks are shown.
            </p>
            <div className="overflow-x-auto">
                <table className="min-w-full text-sm">
                    <thead>
                        <tr className="border-b border-gray-200 text-left text-gray-600">
                            <th className="py-2 pr-4 font-medium w-12">#</th>
                            <SortHeader
                                label="Start date"
                                sortKey="start"
                                current={sort}
                                onSort={handleSort}
                                align="left"
                            />
                            <th className="py-2 pr-4 font-medium text-left text-gray-600">
                                End date
                            </th>
                            <SortHeader
                                label="Days w/ comics"
                                sortKey="days"
                                current={sort}
                                onSort={handleSort}
                            />
                            <SortHeader
                                label="Calendar days"
                                sortKey="calendar"
                                current={sort}
                                onSort={handleSort}
                            />
                        </tr>
                    </thead>
                    <tbody>
                        {sorted.map((row, i) => (
                            <tr
                                key={row.streakStart}
                                className="border-b border-gray-100 hover:bg-gray-50"
                            >
                                <td className="py-2 pr-4 text-gray-400">
                                    {i + 1}
                                </td>
                                <td className="py-2 pr-4 font-medium text-gray-900">
                                    {row.streakStart}
                                </td>
                                <td className="py-2 pr-4 text-gray-700">
                                    {row.streakEnd}
                                </td>
                                <td className="py-2 pr-4 text-right font-medium text-indigo-700">
                                    {row.daysWithComics.toLocaleString()}
                                </td>
                                <td className="py-2 text-right text-gray-500">
                                    {row.calendarDays.toLocaleString()}
                                </td>
                            </tr>
                        ))}
                    </tbody>
                </table>
            </div>
        </div>
    );
}
