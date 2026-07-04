import type { MilestoneComic } from 'models/MilestoneComic';
import { useEffect, useMemo, useState } from 'react';

import { getStatsMilestones } from 'bindings/api/GetStatsMilestones';

import {
    SortableHeader,
    StaticHeader,
    StatsTable,
    StatsTbodyRow,
    StatsTheadRow,
    useSortState,
} from './StatsTable';

type SortKey = 'comicId' | 'title' | 'pubDate';

export default function MilestoneTracker() {
    const [data, setData] = useState<MilestoneComic[] | null>(null);
    const [error, setError] = useState<string | null>(null);
    const [sort, handleSort] = useSortState<SortKey>('comicId', 'asc');

    useEffect(() => {
        getStatsMilestones()
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
                sort.key === 'title'
                    ? (a.title ?? '').localeCompare(b.title ?? '')
                    : sort.key === 'pubDate'
                      ? (a.pubDate ?? '').localeCompare(b.pubDate ?? '')
                      : a.comicId - b.comicId;
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
                Milestone Tracker
            </h2>
            <p className="text-sm text-gray-500 mb-4">
                Round-number comics that serve as milestones in the archive: #1
                (the beginning), every 100th up to #1000, and every 500th beyond
                that.
            </p>
            <StatsTable>
                <thead>
                    <StatsTheadRow>
                        <SortableHeader
                            sortKey="comicId"
                            sort={sort}
                            onSort={handleSort}
                            align="left"
                        >
                            #
                        </SortableHeader>
                        <SortableHeader
                            sortKey="title"
                            sort={sort}
                            onSort={handleSort}
                            align="left"
                        >
                            Title
                        </SortableHeader>
                        <SortableHeader
                            sortKey="pubDate"
                            sort={sort}
                            onSort={handleSort}
                            align="left"
                        >
                            Date
                        </SortableHeader>
                        <StaticHeader>Flags</StaticHeader>
                    </StatsTheadRow>
                </thead>
                <tbody>
                    {sorted.map((row) => (
                        <StatsTbodyRow key={row.comicId}>
                            <td className="py-2 pr-4 font-medium text-indigo-700">
                                #{row.comicId}
                            </td>
                            <td className="py-2 pr-4 text-gray-900">
                                {row.title || (
                                    <span className="text-gray-400 italic">
                                        Untitled
                                    </span>
                                )}
                            </td>
                            <td className="py-2 pr-4 text-gray-500">
                                {row.pubDate ?? '—'}
                            </td>
                            <td className="py-2 text-gray-500">
                                {row.isGuestComic && (
                                    <span className="inline-block mr-1 px-1.5 py-0.5 rounded bg-yellow-100 text-yellow-700 text-xs">
                                        Guest
                                    </span>
                                )}
                                {row.isNonCanon && (
                                    <span className="inline-block px-1.5 py-0.5 rounded bg-gray-100 text-gray-500 text-xs">
                                        Non-canon
                                    </span>
                                )}
                            </td>
                        </StatsTbodyRow>
                    ))}
                </tbody>
            </StatsTable>
        </div>
    );
}
