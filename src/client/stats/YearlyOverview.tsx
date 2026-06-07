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

import type { YearlyOverview } from '../../../bindings/YearlyOverview';

Chart.register(
    BarController,
    BarElement,
    CategoryScale,
    LinearScale,
    Tooltip,
    Legend,
);

interface Props {
    data: YearlyOverview[];
}

function OverviewChart({ data }: Props) {
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
                        label: 'Returning cast',
                        data: data.map((d) => d.totalCast - d.newCast),
                        backgroundColor: 'rgba(99, 102, 241, 0.7)',
                        stack: 'cast',
                    },
                    {
                        label: 'New debuts',
                        data: data.map((d) => d.newCast),
                        backgroundColor: 'rgba(74, 222, 128, 0.7)',
                        stack: 'cast',
                    },
                ],
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                animation: false,
                plugins: {
                    legend: { position: 'bottom' },
                    tooltip: {
                        callbacks: {
                            label: (ctx) =>
                                `${ctx.dataset.label}: ${(ctx.parsed.y as number).toLocaleString()}`,
                            footer: (items) => {
                                const total = items.reduce(
                                    (sum, item) =>
                                        sum + (item.parsed.y as number),
                                    0,
                                );
                                return `Total cast: ${total.toLocaleString()}`;
                            },
                        },
                    },
                },
                scales: {
                    x: { stacked: true },
                    y: {
                        stacked: true,
                        beginAtZero: true,
                        title: { display: true, text: 'Distinct characters' },
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
        <div style={{ height: `${Math.max(300, data.length * 14)}px` }}>
            <canvas ref={canvasRef} />
        </div>
    );
}

export default function YearlyOverviewPage() {
    const [data, setData] = useState<YearlyOverview[] | null>(null);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        fetch('/api/v3/stats/yearly-overview')
            .then((r) => {
                if (!r.ok) throw new Error(`HTTP ${r.status}`);
                return r.json() as Promise<YearlyOverview[]>;
            })
            .then(setData)
            .catch((e: unknown) =>
                setError(e instanceof Error ? e.message : String(e)),
            );
    }, []);

    if (error) {
        return <p className="text-red-600">Failed to load data: {error}</p>;
    }

    return (
        <div>
            <h2 className="text-xl font-semibold text-gray-800 mb-1">
                Yearly Cast Overview
            </h2>
            <p className="text-sm text-gray-500 mb-4">
                How many distinct characters appeared each year, split between
                new debuts and returning cast members.
            </p>
            {!data ? (
                <p className="text-gray-500">Loading…</p>
            ) : (
                <OverviewChart data={data} />
            )}
        </div>
    );
}
