import { useEffect, useMemo, useState } from 'react';

import type { PublicationGap } from '../../../bindings/PublicationGap';
import {
    SortableHeader,
    StaticHeader,
    StatsTable,
    StatsTbodyRow,
    StatsTheadRow,
    comicLink,
    useSortState,
} from './StatsTable';

type SortKey = 'beforeComic' | 'afterComic' | 'gapDays';

function formatGap(days: number) {
    if (days >= 365) {
        const years = (days / 365).toFixed(1);
        return `${years}y (${days.toLocaleString()}d)`;
    }
    return `${days.toLocaleString()} days`;
}

export default function PublicationGaps() {
    const [data, setData] = useState<PublicationGap[] | null>(null);
    const [error, setError] = useState<string | null>(null);
    const [sort, handleSort] = useSortState<SortKey>('gapDays', 'desc');

    useEffect(() => {
        fetch('/api/v3/stats/publication-gaps')
            .then((r) => {
                if (!r.ok) throw new Error(`HTTP ${r.status}`);
                return r.json() as Promise<PublicationGap[]>;
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
                sort.key === 'beforeComic'
                    ? a.beforeComic - b.beforeComic
                    : sort.key === 'afterComic'
                      ? a.afterComic - b.afterComic
                      : a.gapDays - b.gapDays;
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
                Publication Gaps
            </h2>
            <p className="text-sm text-gray-500 mb-4">
                The 20 longest hiatuses between consecutive published comics
                (minimum 7 days apart).
            </p>
            <StatsTable>
                <thead>
                    <StatsTheadRow>
                        <StaticHeader className="w-12">#</StaticHeader>
                        <SortableHeader
                            sortKey="beforeComic"
                            sort={sort}
                            onSort={handleSort}
                        >
                            Last comic before
                        </SortableHeader>
                        <SortableHeader
                            sortKey="afterComic"
                            sort={sort}
                            onSort={handleSort}
                        >
                            First comic after
                        </SortableHeader>
                        <SortableHeader
                            sortKey="gapDays"
                            sort={sort}
                            onSort={handleSort}
                        >
                            Gap length
                        </SortableHeader>
                    </StatsTheadRow>
                </thead>
                <tbody>
                    {sorted.map((row, i) => (
                        <StatsTbodyRow
                            key={`${row.beforeComic}-${row.afterComic}`}
                        >
                            <td className="py-2 pr-4 text-gray-400">{i + 1}</td>
                            <td className="py-2 pr-4 text-right">
                                <a
                                    href={comicLink(row.beforeComic)}
                                    className="text-blue-600 hover:underline"
                                    target="_blank"
                                    rel="noreferrer"
                                >
                                    #{row.beforeComic}
                                </a>
                            </td>
                            <td className="py-2 pr-4 text-right">
                                <a
                                    href={comicLink(row.afterComic)}
                                    className="text-blue-600 hover:underline"
                                    target="_blank"
                                    rel="noreferrer"
                                >
                                    #{row.afterComic}
                                </a>
                            </td>
                            <td className="py-2 text-right text-gray-700">
                                {formatGap(row.gapDays)}
                            </td>
                        </StatsTbodyRow>
                    ))}
                </tbody>
            </StatsTable>
        </div>
    );
}
