import { useEffect, useMemo, useState } from 'react';

import type { CharacterHomeTurfEntry } from 'bindings/CharacterHomeTurfEntry';
import { getStatsCharacterHomeTurf } from 'bindings/api/GetStatsCharacterHomeTurf';

import ItemDetailsModal from './ItemDetailsModal';
import {
    SortableHeader,
    StaticHeader,
    StatsTable,
    StatsTbodyRow,
    StatsTheadRow,
    useSortState,
} from './StatsTable';

type SortKey = 'together' | 'pct' | 'character' | 'location';

export default function CharacterHomeTurf() {
    const [data, setData] = useState<CharacterHomeTurfEntry[] | null>(null);
    const [error, setError] = useState<string | null>(null);
    const [sort, handleSort] = useSortState<SortKey>('pct', 'desc');
    const [selectedItemId, setSelectedItemId] = useState<number | null>(null);

    useEffect(() => {
        getStatsCharacterHomeTurf()
            .then(setData)
            .catch((e: unknown) =>
                setError(e instanceof Error ? e.message : String(e)),
            );
    }, []);

    const sorted = useMemo(() => {
        if (!data) return null;
        const copy = [...data];
        copy.sort((a, b) => {
            const togetherA = a.locations[0]?.comicsTogether ?? 0;
            const togetherB = b.locations[0]?.comicsTogether ?? 0;
            const pctA =
                a.characterAppearances > 0
                    ? togetherA / a.characterAppearances
                    : 0;
            const pctB =
                b.characterAppearances > 0
                    ? togetherB / b.characterAppearances
                    : 0;
            const diff =
                sort.key === 'together'
                    ? togetherA - togetherB
                    : sort.key === 'pct'
                      ? pctA - pctB
                      : sort.key === 'location'
                        ? (a.locations[0]?.locationName ?? '').localeCompare(
                              b.locations[0]?.locationName ?? '',
                          )
                        : a.characterName.localeCompare(b.characterName);
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
                    Character Home Turf
                </h2>
                <p className="text-sm text-gray-500 mb-4">
                    Each character&apos;s most frequently visited location and
                    what percentage of their appearances take place there. Only
                    characters with at least 10 appearances are shown.
                </p>
                <StatsTable>
                    <thead>
                        <StatsTheadRow>
                            <StaticHeader className="w-12">#</StaticHeader>
                            <SortableHeader
                                sortKey="character"
                                sort={sort}
                                onSort={handleSort}
                                align="left"
                            >
                                Character
                            </SortableHeader>
                            <SortableHeader
                                sortKey="location"
                                sort={sort}
                                onSort={handleSort}
                                align="left"
                            >
                                Home location
                            </SortableHeader>
                            <SortableHeader
                                sortKey="together"
                                sort={sort}
                                onSort={handleSort}
                            >
                                Comics together
                            </SortableHeader>
                            <SortableHeader
                                sortKey="pct"
                                sort={sort}
                                onSort={handleSort}
                            >
                                % of appearances
                            </SortableHeader>
                        </StatsTheadRow>
                    </thead>
                    <tbody>
                        {sorted.map((row, i) => {
                            const together =
                                row.locations[0]?.comicsTogether ?? 0;
                            const pct =
                                row.characterAppearances > 0
                                    ? (together / row.characterAppearances) *
                                      100
                                    : 0;
                            return (
                                <StatsTbodyRow key={row.characterId}>
                                    <td className="py-2 pr-4 text-gray-400">
                                        {i + 1}
                                    </td>
                                    <td className="py-2 pr-4">
                                        <button
                                            type="button"
                                            onClick={() => {
                                                setSelectedItemId(
                                                    row.characterId,
                                                );
                                            }}
                                            className="font-medium text-gray-900 hover:text-blue-600 hover:underline text-left"
                                        >
                                            {row.characterName}
                                        </button>
                                    </td>
                                    <td className="py-2 pr-4">
                                        {row.locations.map((loc) => (
                                            <div key={loc.locationId}>
                                                <button
                                                    type="button"
                                                    onClick={() => {
                                                        setSelectedItemId(
                                                            loc.locationId,
                                                        );
                                                    }}
                                                    className="text-gray-700 hover:text-blue-600 hover:underline text-left"
                                                >
                                                    {loc.locationName}
                                                </button>
                                            </div>
                                        ))}
                                    </td>
                                    <td className="py-2 pr-4 text-right text-gray-500">
                                        {together.toLocaleString()}
                                    </td>
                                    <td
                                        className={`py-2 text-right font-medium ${pct >= 50 ? 'text-green-600' : pct >= 25 ? 'text-indigo-700' : 'text-gray-500'}`}
                                    >
                                        {pct.toFixed(1)}%
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
