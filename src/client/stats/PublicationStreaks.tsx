import { useEffect, useMemo, useState } from 'react';

import type { PublicationStreak } from '../../../bindings/PublicationStreak';
import {
    SortableHeader,
    StaticHeader,
    StatsTable,
    StatsTbodyRow,
    StatsTheadRow,
    useSortState,
} from './StatsTable';

type SortKey = 'days' | 'calendar' | 'start';

function formatBreakDate(breakDate: string | null): string {
    if (breakDate === null) return 'Ongoing';
    const date = new Date(`${breakDate}T00:00:00`);
    return `Missed ${date.toLocaleDateString('en-US', { weekday: 'long', month: 'short', day: 'numeric', year: 'numeric' })}`;
}

export default function PublicationStreaks() {
    const [data, setData] = useState<PublicationStreak[] | null>(null);
    const [error, setError] = useState<string | null>(null);
    const [sort, handleSort] = useSortState<SortKey>('days', 'desc');

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
            <StatsTable>
                <thead>
                    <StatsTheadRow>
                        <StaticHeader className="w-12">#</StaticHeader>
                        <SortableHeader
                            sortKey="start"
                            sort={sort}
                            onSort={handleSort}
                            align="left"
                        >
                            Start date
                        </SortableHeader>
                        <StaticHeader align="left">End date</StaticHeader>
                        <SortableHeader
                            sortKey="days"
                            sort={sort}
                            onSort={handleSort}
                        >
                            Days w/ comics
                        </SortableHeader>
                        <SortableHeader
                            sortKey="calendar"
                            sort={sort}
                            onSort={handleSort}
                        >
                            Calendar days
                        </SortableHeader>
                        <StaticHeader align="left">Ended</StaticHeader>
                    </StatsTheadRow>
                </thead>
                <tbody>
                    {sorted.map((row, i) => (
                        <StatsTbodyRow key={row.streakStart}>
                            <td className="py-2 pr-4 text-gray-400">{i + 1}</td>
                            <td className="py-2 pr-4 font-medium text-gray-900">
                                {row.streakStart}
                            </td>
                            <td className="py-2 pr-4 text-gray-700">
                                {row.streakEnd}
                            </td>
                            <td className="py-2 pr-4 text-right font-medium text-indigo-700">
                                {row.daysWithComics.toLocaleString()}
                            </td>
                            <td className="py-2 pr-4 text-right text-gray-500">
                                {row.calendarDays.toLocaleString()}
                            </td>
                            <td
                                className={`py-2 text-sm ${row.breakDate === null ? 'font-medium text-green-600' : 'text-gray-500'}`}
                            >
                                {formatBreakDate(row.breakDate)}
                            </td>
                        </StatsTbodyRow>
                    ))}
                </tbody>
            </StatsTable>
        </div>
    );
}
