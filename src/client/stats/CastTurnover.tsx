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

import type { CastTurnoverYear } from 'bindings/CastTurnoverYear';
import { getStatsCastTurnover } from 'bindings/api/GetStatsCastTurnover';

Chart.register(
    BarController,
    BarElement,
    CategoryScale,
    LinearScale,
    Tooltip,
    Legend,
);

interface ChartProps {
    data: CastTurnoverYear[];
}

function TurnoverChart({ data }: ChartProps) {
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
                        label: 'New',
                        data: data.map((d) => d.newChars),
                        backgroundColor: 'rgba(34, 197, 94, 0.8)',
                        stack: 'a',
                    },
                    {
                        label: 'Continuing',
                        data: data.map((d) => d.continuingChars),
                        backgroundColor: 'rgba(99, 102, 241, 0.8)',
                        stack: 'a',
                    },
                    {
                        label: 'Returning',
                        data: data.map((d) => d.returningChars),
                        backgroundColor: 'rgba(251, 191, 36, 0.8)',
                        stack: 'a',
                    },
                    {
                        label: 'Dropped',
                        data: data.map((d) => d.droppedChars),
                        backgroundColor: 'rgba(239, 68, 68, 0.8)',
                        stack: 'b',
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
                                `${ctx.dataset.label ?? ''}: ${(ctx.parsed.y as number).toLocaleString()} characters`,
                        },
                    },
                },
                scales: {
                    x: { stacked: true },
                    y: {
                        stacked: true,
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
    }, [data]);

    return (
        <div style={{ height: '400px' }}>
            <canvas ref={canvasRef} />
        </div>
    );
}

export default function CastTurnover() {
    const [data, setData] = useState<CastTurnoverYear[] | null>(null);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        getStatsCastTurnover()
            .then(setData)
            .catch((e: unknown) =>
                setError(e instanceof Error ? e.message : String(e)),
            );
    }, []);

    if (error)
        return <p className="text-red-600">Failed to load data: {error}</p>;

    return (
        <div>
            <h2 className="text-xl font-semibold text-gray-800 mb-1">
                Cast Turnover
            </h2>
            <p className="text-sm text-gray-500 mb-4">
                Year-by-year breakdown of cast composition: new characters
                (first appearance ever), continuing characters (also appeared
                the prior year), returning characters (absent at least one
                year), and characters dropped (appeared last year but not this
                one). &ldquo;Dropped&rdquo; is shown on a separate stack for
                clarity.
            </p>
            {!data ? (
                <p className="text-gray-500">Loading…</p>
            ) : (
                <TurnoverChart data={data} />
            )}
        </div>
    );
}
