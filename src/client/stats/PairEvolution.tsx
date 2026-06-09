import {
    BarController,
    BarElement,
    CategoryScale,
    Chart,
    Legend,
    LinearScale,
    Tooltip,
} from 'chart.js';
import { useEffect, useRef, useState } from 'react';

import type { ItemStats } from 'bindings/ItemStats';
import type { PairEvolutionYear } from 'bindings/PairEvolutionYear';
import { getStatsPairEvolution } from 'bindings/api/GetStatsPairEvolution';

Chart.register(
    BarController,
    BarElement,
    CategoryScale,
    LinearScale,
    Tooltip,
    Legend,
);

interface ChartProps {
    data: PairEvolutionYear[];
    name1: string;
    name2: string;
}

function PairChart({ data, name1, name2 }: ChartProps) {
    const canvasRef = useRef<HTMLCanvasElement>(null);
    const chartRef = useRef<Chart | null>(null);

    useEffect(() => {
        if (!canvasRef.current) return;
        chartRef.current?.destroy();

        chartRef.current = new Chart(canvasRef.current, {
            type: 'bar',
            data: {
                labels: data.map((d) => String(d.year)),
                datasets: [
                    {
                        label: `${name1} & ${name2}`,
                        data: data.map((d) => d.comicsTogether),
                        backgroundColor: 'rgba(99, 102, 241, 0.8)',
                    },
                ],
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                animation: false,
                plugins: {
                    legend: { display: false },
                    tooltip: {
                        callbacks: {
                            label: (ctx) =>
                                `${(ctx.parsed.y as number).toLocaleString()} comics together`,
                        },
                    },
                },
                scales: {
                    x: {},
                    y: {
                        beginAtZero: true,
                        title: { display: true, text: 'Comics together' },
                    },
                },
            },
        });

        return () => {
            chartRef.current?.destroy();
            chartRef.current = null;
        };
    }, [data, name1, name2]);

    return (
        <div style={{ height: '300px' }}>
            <canvas ref={canvasRef} />
        </div>
    );
}

interface Props {
    castData: ItemStats[] | null;
    castError: string | null;
}

export default function PairEvolution({ castData, castError }: Props) {
    const [char1Override, setChar1Override] = useState<number | null>(null);
    const [char2Override, setChar2Override] = useState<number | null>(null);
    const [pairData, setPairData] = useState<PairEvolutionYear[] | null>(null);
    const [fetchError, setFetchError] = useState<string | null>(null);

    // Derive effective IDs from user override or defaults from castData
    const char1Id = char1Override ?? castData?.[0]?.id ?? null;
    const char2Id = char2Override ?? castData?.[1]?.id ?? null;

    useEffect(() => {
        if (char1Id === null || char2Id === null || char1Id === char2Id) return;
        const controller = new AbortController();
        getStatsPairEvolution(
            { char1: char1Id, char2: char2Id },
            { signal: controller.signal },
        )
            .then((d) => {
                setPairData(d);
                setFetchError(null);
            })
            .catch((e: unknown) => {
                if (controller.signal.aborted) return;
                setFetchError(e instanceof Error ? e.message : String(e));
            });
        return () => {
            controller.abort();
        };
    }, [char1Id, char2Id]);

    const error = castError ?? fetchError;
    if (error)
        return <p className="text-red-600">Failed to load data: {error}</p>;
    if (!castData) return <p className="text-gray-500">Loading…</p>;

    const char1 = castData.find((c) => c.id === char1Id);
    const char2 = castData.find((c) => c.id === char2Id);

    return (
        <div>
            <h2 className="text-xl font-semibold text-gray-800 mb-1">
                Pair Evolution
            </h2>
            <p className="text-sm text-gray-500 mb-4">
                How often two characters have appeared together, broken down by
                year. Select any two cast members to see when their relationship
                was most active.
            </p>
            <div className="flex flex-wrap gap-4 mb-4">
                <div>
                    <label
                        htmlFor="pair-char1"
                        className="block text-sm font-medium text-gray-700 mb-1"
                    >
                        Character 1
                    </label>
                    <select
                        id="pair-char1"
                        className="border border-gray-300 rounded-md px-3 py-1.5 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500"
                        value={char1Id ?? ''}
                        onChange={(e) => {
                            setChar1Override(Number(e.target.value));
                        }}
                    >
                        {castData.map((c) => (
                            <option
                                key={c.id}
                                value={c.id}
                                disabled={c.id === char2Id}
                            >
                                {c.name}
                            </option>
                        ))}
                    </select>
                </div>
                <div>
                    <label
                        htmlFor="pair-char2"
                        className="block text-sm font-medium text-gray-700 mb-1"
                    >
                        Character 2
                    </label>
                    <select
                        id="pair-char2"
                        className="border border-gray-300 rounded-md px-3 py-1.5 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500"
                        value={char2Id ?? ''}
                        onChange={(e) => {
                            setChar2Override(Number(e.target.value));
                        }}
                    >
                        {castData.map((c) => (
                            <option
                                key={c.id}
                                value={c.id}
                                disabled={c.id === char1Id}
                            >
                                {c.name}
                            </option>
                        ))}
                    </select>
                </div>
            </div>
            {char1Id === char2Id && (
                <p className="text-amber-600 text-sm mb-4">
                    Please select two different characters.
                </p>
            )}
            {pairData === null && char1Id !== char2Id ? (
                <p className="text-gray-500">Loading…</p>
            ) : pairData && pairData.length === 0 ? (
                <p className="text-gray-500">
                    These two characters have never appeared together.
                </p>
            ) : pairData ? (
                <PairChart
                    data={pairData}
                    name1={char1?.name ?? ''}
                    name2={char2?.name ?? ''}
                />
            ) : null}
        </div>
    );
}
