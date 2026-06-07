import {
    CategoryScale,
    Chart,
    Legend,
    LineController,
    LineElement,
    LinearScale,
    PointElement,
    Tooltip,
} from 'chart.js';
import { useEffect, useRef, useState } from 'react';

import type { CrowdedComicsResponse } from '../../../bindings/CrowdedComicsResponse';

Chart.register(
    LineController,
    LineElement,
    PointElement,
    CategoryScale,
    LinearScale,
    Tooltip,
    Legend,
);

function comicLink(comicId: number) {
    return `https://questionablecontent.net/view.php?comic=${comicId}`;
}

interface ChartProps {
    data: CrowdedComicsResponse;
}

function AvgCastChart({ data }: ChartProps) {
    const canvasRef = useRef<HTMLCanvasElement>(null);
    const chartRef = useRef<Chart | null>(null);

    useEffect(() => {
        if (!canvasRef.current) return;
        chartRef.current?.destroy();

        chartRef.current = new Chart(canvasRef.current, {
            type: 'line',
            data: {
                labels: data.avgPerYear.map((d) => String(d.year)),
                datasets: [
                    {
                        label: 'Average cast size',
                        data: data.avgPerYear.map((d) => d.avgCastSize),
                        borderColor: 'rgba(99, 102, 241, 0.8)',
                        backgroundColor: 'rgba(99, 102, 241, 0.1)',
                        borderWidth: 2,
                        pointRadius: 2,
                        fill: true,
                        tension: 0.3,
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
                                `Avg cast: ${(ctx.parsed.y as number).toFixed(2)}`,
                        },
                    },
                },
                scales: {
                    x: { title: { display: true, text: 'Year' } },
                    y: {
                        beginAtZero: true,
                        title: { display: true, text: 'Avg cast per comic' },
                    },
                },
            },
        });

        return () => {
            chartRef.current?.destroy();
            chartRef.current = null;
        };
    }, [data]);

    return (
        <div style={{ height: '280px' }}>
            <canvas ref={canvasRef} />
        </div>
    );
}

export default function MostCrowdedComics() {
    const [data, setData] = useState<CrowdedComicsResponse | null>(null);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        fetch('/api/v3/stats/crowded-comics')
            .then((r) => {
                if (!r.ok) throw new Error(`HTTP ${r.status}`);
                return r.json() as Promise<CrowdedComicsResponse>;
            })
            .then(setData)
            .catch((e: unknown) =>
                setError(e instanceof Error ? e.message : String(e)),
            );
    }, []);

    if (error) {
        return <p className="text-red-600">Failed to load data: {error}</p>;
    }

    if (!data) {
        return <p className="text-gray-500">Loading…</p>;
    }

    return (
        <div>
            <h2 className="text-xl font-semibold text-gray-800 mb-1">
                Most Crowded Comics
            </h2>
            <p className="text-sm text-gray-500 mb-6">
                The 25 comics with the most cast members tagged, and how the
                average cast size per comic has trended over the years.
            </p>

            <h3 className="text-base font-medium text-gray-700 mb-2">
                Average cast size per year
            </h3>
            <div className="mb-8">
                <AvgCastChart data={data} />
            </div>

            <h3 className="text-base font-medium text-gray-700 mb-2">
                Top 25 most crowded comics
            </h3>
            <div className="overflow-x-auto">
                <table className="min-w-full text-sm">
                    <thead>
                        <tr className="border-b border-gray-200 text-left text-gray-600">
                            <th className="py-2 pr-4 font-medium w-12">#</th>
                            <th className="py-2 pr-4 font-medium">Comic</th>
                            <th className="py-2 font-medium text-right">
                                Cast members
                            </th>
                        </tr>
                    </thead>
                    <tbody>
                        {data.topComics.map((row, i) => (
                            <tr
                                key={row.comicId}
                                className="border-b border-gray-100 hover:bg-gray-50"
                            >
                                <td className="py-2 pr-4 text-gray-400">
                                    {i + 1}
                                </td>
                                <td className="py-2 pr-4">
                                    <a
                                        href={comicLink(row.comicId)}
                                        className="text-blue-600 hover:underline"
                                        target="_blank"
                                        rel="noreferrer"
                                    >
                                        #{row.comicId}
                                    </a>
                                </td>
                                <td className="py-2 text-right text-gray-700">
                                    {row.castCount}
                                </td>
                            </tr>
                        ))}
                    </tbody>
                </table>
            </div>
        </div>
    );
}
