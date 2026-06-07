import { useMemo, useState } from 'react';

import type { ItemStats } from '../../../bindings/ItemStats';

function comicLink(comicId: number) {
    return `https://questionablecontent.net/view.php?comic=${comicId}`;
}

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

    const { retired, maxComic } = useMemo(() => {
        if (!sharedData || sharedData.length === 0)
            return { retired: null, maxComic: 0 };
        const max = Math.max(...sharedData.map((d) => d.lastComic));
        const cutoff = max - threshold;
        const filtered = sharedData
            .filter((d) => d.lastComic < cutoff)
            .sort((a, b) => a.lastComic - b.lastComic);
        return { retired: filtered, maxComic: max };
    }, [sharedData, threshold]);

    if (sharedError) {
        return (
            <p className="text-red-600">Failed to load data: {sharedError}</p>
        );
    }

    if (!retired) {
        return <p className="text-gray-500">Loading…</p>;
    }

    return (
        <div>
            <h2 className="text-xl font-semibold text-gray-800 mb-1">
                Retired Characters
            </h2>
            <div className="flex items-center gap-4 mb-4">
                <p className="text-sm text-gray-500">
                    Characters who haven't appeared in the last{' '}
                    <strong>{threshold.toLocaleString()}</strong> comics (before
                    #{(maxComic - threshold).toLocaleString()}).{' '}
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
            <div className="overflow-x-auto">
                <table className="min-w-full text-sm">
                    <thead>
                        <tr className="border-b border-gray-200 text-left text-gray-600">
                            <th className="py-2 pr-4 font-medium w-12">#</th>
                            <th className="py-2 pr-4 font-medium">Name</th>
                            <th className="py-2 pr-4 font-medium text-right">
                                Appearances
                            </th>
                            <th className="py-2 pr-4 font-medium text-right">
                                First comic
                            </th>
                            <th className="py-2 font-medium text-right">
                                Last comic
                            </th>
                        </tr>
                    </thead>
                    <tbody>
                        {retired.map((row, i) => (
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
                            </tr>
                        ))}
                    </tbody>
                </table>
            </div>
        </div>
    );
}
