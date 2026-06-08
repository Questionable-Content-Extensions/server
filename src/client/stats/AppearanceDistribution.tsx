import {
    BarController,
    BarElement,
    CategoryScale,
    Chart,
    Legend,
    LinearScale,
    Tooltip,
} from 'chart.js';
import { useEffect, useMemo, useRef } from 'react';

import type { ItemStats } from '../../../bindings/ItemStats';
import {
    StaticHeader,
    StatsTable,
    StatsTbodyRow,
    StatsTheadRow,
} from './StatsTable';

Chart.register(
    BarController,
    BarElement,
    CategoryScale,
    LinearScale,
    Tooltip,
    Legend,
);

interface Bucket {
    label: string;
    min: number;
    max: number;
}

const BUCKETS: Bucket[] = [
    { label: '1', min: 1, max: 1 },
    { label: '2–5', min: 2, max: 5 },
    { label: '6–10', min: 6, max: 10 },
    { label: '11–25', min: 11, max: 25 },
    { label: '26–50', min: 26, max: 50 },
    { label: '51–100', min: 51, max: 100 },
    { label: '101–200', min: 101, max: 200 },
    { label: '201–500', min: 201, max: 500 },
    { label: '501+', min: 501, max: Infinity },
];

interface ChartProps {
    counts: number[];
}

function DistributionChart({ counts }: ChartProps) {
    const canvasRef = useRef<HTMLCanvasElement>(null);
    const chartRef = useRef<Chart | null>(null);

    useEffect(() => {
        if (!canvasRef.current) return;
        chartRef.current?.destroy();

        chartRef.current = new Chart(canvasRef.current, {
            type: 'bar',
            data: {
                labels: BUCKETS.map((b) => b.label),
                datasets: [
                    {
                        label: 'Characters',
                        data: counts,
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
                                `${(ctx.parsed.y as number).toLocaleString()} characters`,
                        },
                    },
                },
                scales: {
                    x: { title: { display: true, text: 'Appearances' } },
                    y: {
                        beginAtZero: true,
                        title: { display: true, text: 'Characters' },
                    },
                },
            },
        });

        return () => {
            chartRef.current?.destroy();
            chartRef.current = null;
        };
    }, [counts]);

    return (
        <div style={{ height: '350px' }}>
            <canvas ref={canvasRef} />
        </div>
    );
}

interface Props {
    castData: ItemStats[] | null;
    castError: string | null;
}

export default function AppearanceDistribution({ castData, castError }: Props) {
    const bucketCounts = useMemo(() => {
        if (!castData) return null;
        return BUCKETS.map(
            (b) =>
                castData.filter(
                    (c) => c.appearances >= b.min && c.appearances <= b.max,
                ).length,
        );
    }, [castData]);

    if (castError)
        return <p className="text-red-600">Failed to load data: {castError}</p>;
    if (!bucketCounts) return <p className="text-gray-500">Loading…</p>;

    const total = castData?.length ?? 0;

    return (
        <div>
            <h2 className="text-xl font-semibold text-gray-800 mb-1">
                Appearance Distribution
            </h2>
            <p className="text-sm text-gray-500 mb-4">
                Histogram of cast members by their total appearance count. Shows
                how many characters are one-timers vs. recurring regulars.
                Total: {total.toLocaleString()} characters.
            </p>
            <DistributionChart counts={bucketCounts} />
            <div className="mt-4">
                <StatsTable>
                    <thead>
                        <StatsTheadRow>
                            <StaticHeader align="left">
                                Appearances
                            </StaticHeader>
                            <StaticHeader align="right">
                                Characters
                            </StaticHeader>
                            <StaticHeader align="right">% of cast</StaticHeader>
                        </StatsTheadRow>
                    </thead>
                    <tbody>
                        {BUCKETS.map((b, i) => (
                            <StatsTbodyRow key={b.label}>
                                <td className="py-2 pr-4 text-gray-700">
                                    {b.label}
                                </td>
                                <td className="py-2 pr-4 text-right font-medium text-indigo-700">
                                    {bucketCounts[i].toLocaleString()}
                                </td>
                                <td className="py-2 text-right text-gray-500">
                                    {total > 0
                                        ? (
                                              (bucketCounts[i] / total) *
                                              100
                                          ).toFixed(1)
                                        : '0.0'}
                                    %
                                </td>
                            </StatsTbodyRow>
                        ))}
                    </tbody>
                </StatsTable>
            </div>
        </div>
    );
}
