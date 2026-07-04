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
import type { CrowdedComicsResponse } from 'models/CrowdedComicsResponse';
import { useEffect, useMemo, useRef, useState } from 'react';

import { getStatsCrowdedComics } from 'bindings/api/GetStatsCrowdedComics';

import {
    SortableHeader,
    StaticHeader,
    StatsTable,
    StatsTbodyRow,
    StatsTheadRow,
    comicLink,
    useSortState,
} from './StatsTable';

Chart.register(
    LineController,
    LineElement,
    PointElement,
    CategoryScale,
    LinearScale,
    Tooltip,
    Legend,
);

type SortKey = 'comicId' | 'castCount';

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
    const [sort, handleSort] = useSortState<SortKey>('castCount', 'desc');

    useEffect(() => {
        getStatsCrowdedComics()
            .then(setData)
            .catch((e: unknown) =>
                setError(e instanceof Error ? e.message : String(e)),
            );
    }, []);

    const sorted = useMemo(() => {
        if (!data) return null;
        const copy = [...data.topComics];
        copy.sort((a, b) => {
            const diff =
                sort.key === 'comicId'
                    ? a.comicId - b.comicId
                    : a.castCount - b.castCount;
            return sort.dir === 'asc' ? diff : -diff;
        });
        return copy;
    }, [data, sort]);

    if (error) {
        return <p className="text-red-600">Failed to load data: {error}</p>;
    }

    if (!data || !sorted) {
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
            <StatsTable>
                <thead>
                    <StatsTheadRow>
                        <StaticHeader className="w-12">#</StaticHeader>
                        <SortableHeader
                            sortKey="comicId"
                            sort={sort}
                            onSort={handleSort}
                            align="left"
                        >
                            Comic
                        </SortableHeader>
                        <SortableHeader
                            sortKey="castCount"
                            sort={sort}
                            onSort={handleSort}
                        >
                            Cast members
                        </SortableHeader>
                    </StatsTheadRow>
                </thead>
                <tbody>
                    {sorted.map((row, i) => (
                        <StatsTbodyRow key={row.comicId}>
                            <td className="py-2 pr-4 text-gray-400">{i + 1}</td>
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
                        </StatsTbodyRow>
                    ))}
                </tbody>
            </StatsTable>
        </div>
    );
}
