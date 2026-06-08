import { useEffect, useMemo, useState } from 'react';

import type { ComebackLocation } from '../../../bindings/ComebackLocation';
import ItemDetailsModal from './ItemDetailsModal';
import {
    SortableHeader,
    StaticHeader,
    StatsTable,
    StatsTbodyRow,
    StatsTheadRow,
    comicLink,
    useSortState,
} from './StatsTable';

type SortKey = 'name' | 'lastComic' | 'returnComic' | 'gapDays';

function formatGap(days: number) {
    if (days >= 365) {
        const years = (days / 365).toFixed(1);
        return `${years}y (${days.toLocaleString()}d)`;
    }
    return `${days.toLocaleString()} days`;
}

export default function ComebackLocations() {
    const [data, setData] = useState<ComebackLocation[] | null>(null);
    const [error, setError] = useState<string | null>(null);
    const [sort, handleSort] = useSortState<SortKey>('gapDays', 'desc');
    const [selectedItemId, setSelectedItemId] = useState<number | null>(null);

    useEffect(() => {
        fetch('/api/v3/stats/comeback-locations')
            .then((r) => {
                if (!r.ok) throw new Error(`HTTP ${r.status}`);
                return r.json() as Promise<ComebackLocation[]>;
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
                sort.key === 'name'
                    ? a.name.localeCompare(b.name)
                    : sort.key === 'lastComic'
                      ? a.lastComic - b.lastComic
                      : sort.key === 'returnComic'
                        ? a.returnComic - b.returnComic
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
                    Comeback Locations
                </h2>
                <p className="text-sm text-gray-500 mb-4">
                    Locations with the longest gap between two consecutive
                    appearances — those that disappeared and came back. Top 50
                    by largest single gap (minimum 90 days).
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
                                sortKey="lastComic"
                                sort={sort}
                                onSort={handleSort}
                            >
                                Last seen
                            </SortableHeader>
                            <SortableHeader
                                sortKey="returnComic"
                                sort={sort}
                                onSort={handleSort}
                            >
                                Returned in
                            </SortableHeader>
                            <SortableHeader
                                sortKey="gapDays"
                                sort={sort}
                                onSort={handleSort}
                            >
                                Gap
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
                                <td className="py-2 pr-4 text-right">
                                    <a
                                        href={comicLink(row.lastComic)}
                                        className="text-blue-600 hover:underline"
                                        target="_blank"
                                        rel="noreferrer"
                                    >
                                        #{row.lastComic}
                                    </a>
                                </td>
                                <td className="py-2 pr-4 text-right">
                                    <a
                                        href={comicLink(row.returnComic)}
                                        className="text-blue-600 hover:underline"
                                        target="_blank"
                                        rel="noreferrer"
                                    >
                                        #{row.returnComic}
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
        </>
    );
}
