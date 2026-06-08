import { useMemo, useState } from 'react';

import type { ItemStats } from '../../../bindings/ItemStats';
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

type SortKey = 'name' | 'appearances' | 'firstComic' | 'lastComic';

const THRESHOLD_OPTIONS = [100, 250, 500, 1000, 2000] as const;

interface RetiredCharactersProps {
    sharedData?: ItemStats[] | null;
    sharedError?: string | null;
}

export default function RetiredCharacters({
    sharedData,
    sharedError,
}: RetiredCharactersProps) {
    const [threshold, setThreshold] = useState<number>(500);
    const [sort, handleSort] = useSortState<SortKey>('lastComic', 'asc');
    const [selectedItemId, setSelectedItemId] = useState<number | null>(null);

    const { retired, maxComic } = useMemo(() => {
        if (!sharedData || sharedData.length === 0)
            return { retired: null, maxComic: 0 };
        const max = Math.max(...sharedData.map((d) => d.lastComic));
        const cutoff = max - threshold;
        const filtered = sharedData.filter((d) => d.lastComic < cutoff);
        const copy = [...filtered];
        copy.sort((a, b) => {
            const diff =
                sort.key === 'name'
                    ? a.name.localeCompare(b.name)
                    : sort.key === 'appearances'
                      ? a.appearances - b.appearances
                      : sort.key === 'firstComic'
                        ? a.firstComic - b.firstComic
                        : a.lastComic - b.lastComic;
            return sort.dir === 'asc' ? diff : -diff;
        });
        return { retired: copy, maxComic: max };
    }, [sharedData, threshold, sort]);

    if (sharedError) {
        return (
            <p className="text-red-600">Failed to load data: {sharedError}</p>
        );
    }

    if (!retired) {
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
                    Retired Characters
                </h2>
                <div className="flex items-center gap-4 mb-4">
                    <p className="text-sm text-gray-500">
                        Characters who haven't appeared in the last{' '}
                        <strong>{threshold.toLocaleString()}</strong> comics
                        (before #{(maxComic - threshold).toLocaleString()}).{' '}
                        {retired.length} character
                        {retired.length !== 1 ? 's' : ''} found.
                    </p>
                    <div className="ml-auto flex items-center gap-2 shrink-0">
                        <label
                            htmlFor="threshold"
                            className="text-sm text-gray-600 whitespace-nowrap"
                        >
                            Gap:
                        </label>
                        <select
                            id="threshold"
                            value={threshold}
                            onChange={(e) => {
                                setThreshold(Number(e.target.value));
                            }}
                            className="text-sm border border-gray-300 rounded px-2 py-1"
                        >
                            {THRESHOLD_OPTIONS.map((t) => (
                                <option key={t} value={t}>
                                    {t.toLocaleString()} comics
                                </option>
                            ))}
                        </select>
                    </div>
                </div>
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
                                Appearances
                            </SortableHeader>
                            <SortableHeader
                                sortKey="firstComic"
                                sort={sort}
                                onSort={handleSort}
                            >
                                First comic
                            </SortableHeader>
                            <SortableHeader
                                sortKey="lastComic"
                                sort={sort}
                                onSort={handleSort}
                            >
                                Last comic
                            </SortableHeader>
                        </StatsTheadRow>
                    </thead>
                    <tbody>
                        {retired.map((row, i) => (
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
                                <td className="py-2 pr-4 text-right">
                                    <a
                                        href={comicLink(row.firstComic)}
                                        className="text-blue-600 hover:underline"
                                        target="_blank"
                                        rel="noreferrer"
                                    >
                                        #{row.firstComic}
                                    </a>
                                </td>
                                <td className="py-2 text-right">
                                    <a
                                        href={comicLink(row.lastComic)}
                                        className="text-blue-600 hover:underline"
                                        target="_blank"
                                        rel="noreferrer"
                                    >
                                        #{row.lastComic}
                                    </a>
                                </td>
                            </StatsTbodyRow>
                        ))}
                    </tbody>
                </StatsTable>
            </div>
        </>
    );
}
