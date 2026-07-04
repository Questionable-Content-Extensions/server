import type { LocationSocialHubEntry } from 'models/LocationSocialHubEntry';
import { useEffect, useMemo, useState } from 'react';

import { getStatsLocationSocialHub } from 'bindings/api/GetStatsLocationSocialHub';

import ItemDetailsModal from './ItemDetailsModal';
import {
    SortableHeader,
    StaticHeader,
    StatsTable,
    StatsTbodyRow,
    StatsTheadRow,
    useSortState,
} from './StatsTable';

type SortKey = 'characters' | 'appearances' | 'name';

export default function LocationSocialHub() {
    const [data, setData] = useState<LocationSocialHubEntry[] | null>(null);
    const [error, setError] = useState<string | null>(null);
    const [sort, handleSort] = useSortState<SortKey>('characters', 'desc');
    const [selectedItemId, setSelectedItemId] = useState<number | null>(null);

    useEffect(() => {
        getStatsLocationSocialHub()
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
                sort.key === 'characters'
                    ? a.distinctCharacters - b.distinctCharacters
                    : sort.key === 'appearances'
                      ? a.appearances - b.appearances
                      : a.name.localeCompare(b.name);
            return sort.dir === 'asc' ? diff : -diff;
        });
        return copy;
    }, [data, sort]);

    const maxCharacters = useMemo(
        () => (data ? Math.max(...data.map((d) => d.distinctCharacters)) : 1),
        [data],
    );

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
                    Location Social Hub
                </h2>
                <p className="text-sm text-gray-500 mb-4">
                    Locations ranked by the number of distinct cast members that
                    have appeared there. A location with a high hub score is a
                    meeting point for the widest variety of characters.
                    &ldquo;Relative reach&rdquo; is a percentage showing how
                    many distinct characters this location has hosted compared
                    to the most-visited location (100% = tied for first place).
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
                                sortKey="characters"
                                sort={sort}
                                onSort={handleSort}
                            >
                                Distinct characters
                            </SortableHeader>
                            <SortableHeader
                                sortKey="appearances"
                                sort={sort}
                                onSort={handleSort}
                            >
                                Appearances
                            </SortableHeader>
                            <StaticHeader align="right">
                                Relative reach
                            </StaticHeader>
                        </StatsTheadRow>
                    </thead>
                    <tbody>
                        {sorted.map((row, i) => {
                            const reach =
                                maxCharacters > 0
                                    ? (row.distinctCharacters / maxCharacters) *
                                      100
                                    : 0;
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
                                        {row.distinctCharacters.toLocaleString()}
                                    </td>
                                    <td className="py-2 pr-4 text-right text-gray-500">
                                        {row.appearances.toLocaleString()}
                                    </td>
                                    <td className="py-2 text-right text-gray-500">
                                        {reach.toFixed(0)}%
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
