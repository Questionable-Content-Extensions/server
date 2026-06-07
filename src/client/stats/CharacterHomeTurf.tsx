import { useEffect, useMemo, useState } from 'react';

import type { CharacterHomeTurfEntry } from '../../../bindings/CharacterHomeTurfEntry';
import ItemDetailsModal from './ItemDetailsModal';

type SortKey = 'together' | 'pct' | 'character' | 'location';
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

export default function CharacterHomeTurf() {
    const [data, setData] = useState<CharacterHomeTurfEntry[] | null>(null);
    const [error, setError] = useState<string | null>(null);
    const [sort, setSort] = useState<SortState>({ key: 'pct', dir: 'desc' });
    const [selectedItemId, setSelectedItemId] = useState<number | null>(null);

    useEffect(() => {
        fetch('/api/v3/stats/character-home-turf')
            .then((r) => {
                if (!r.ok) throw new Error(`HTTP ${r.status}`);
                return r.json() as Promise<CharacterHomeTurfEntry[]>;
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
                    what percentage of their appearances take place there. High
                    percentages indicate a character who is strongly tied to a
                    single setting. Only characters with at least 10 appearances
                    are shown.
                </p>
                <div className="overflow-x-auto">
                    <table className="min-w-full text-sm">
                        <thead>
                            <tr className="border-b border-gray-200 text-left text-gray-600">
                                <th className="py-2 pr-4 font-medium w-12">
                                    #
                                </th>
                                <SortHeader
                                    label="Character"
                                    sortKey="character"
                                    current={sort}
                                    onSort={handleSort}
                                    align="left"
                                />
                                <SortHeader
                                    label="Home location"
                                    sortKey="location"
                                    current={sort}
                                    onSort={handleSort}
                                    align="left"
                                />
                                <SortHeader
                                    label="Comics together"
                                    sortKey="together"
                                    current={sort}
                                    onSort={handleSort}
                                />
                                <SortHeader
                                    label="% of appearances"
                                    sortKey="pct"
                                    current={sort}
                                    onSort={handleSort}
                                />
                            </tr>
                        </thead>
                        <tbody>
                            {sorted.map((row, i) => {
                                const together =
                                    row.locations[0]?.comicsTogether ?? 0;
                                const pct =
                                    row.characterAppearances > 0
                                        ? (together /
                                              row.characterAppearances) *
                                          100
                                        : 0;
                                return (
                                    <tr
                                        key={row.characterId}
                                        className="border-b border-gray-100 hover:bg-gray-50"
                                    >
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
                                    </tr>
                                );
                            })}
                        </tbody>
                    </table>
                </div>
            </div>
        </>
    );
}
