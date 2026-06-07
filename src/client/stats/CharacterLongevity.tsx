import { useMemo } from 'react';

import type { ItemStats } from '../../../bindings/ItemStats';

function comicLink(comicId: number) {
    return `https://questionablecontent.net/view.php?comic=${comicId}`;
}

interface CharacterLongevityProps {
    sharedData?: ItemStats[] | null;
    sharedError?: string | null;
}

export default function CharacterLongevity({
    sharedData,
    sharedError,
}: CharacterLongevityProps) {
    const data = useMemo(() => {
        if (!sharedData) return null;
        return [...sharedData].sort(
            (a, b) => b.lastComic - b.firstComic - (a.lastComic - a.firstComic),
        );
    }, [sharedData]);

    if (sharedError) {
        return (
            <p className="text-red-600">Failed to load data: {sharedError}</p>
        );
    }

    if (!data) {
        return <p className="text-gray-500">Loading…</p>;
    }

    return (
        <div>
            <h2 className="text-xl font-semibold text-gray-800 mb-1">
                Character Longevity
            </h2>
            <p className="text-sm text-gray-500 mb-4">
                Characters ranked by the span between their first and last
                appearance (in comic numbers). A large span means a character
                has been part of the story for a long stretch.
            </p>
            <div className="overflow-x-auto">
                <table className="min-w-full text-sm">
                    <thead>
                        <tr className="border-b border-gray-200 text-left text-gray-600">
                            <th className="py-2 pr-4 font-medium w-12">#</th>
                            <th className="py-2 pr-4 font-medium">Name</th>
                            <th className="py-2 pr-4 font-medium text-right">
                                Span
                            </th>
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
                        {data.map((row, i) => {
                            const span = row.lastComic - row.firstComic;
                            return (
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
                                        {span.toLocaleString()}
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
                            );
                        })}
                    </tbody>
                </table>
            </div>
        </div>
    );
}
