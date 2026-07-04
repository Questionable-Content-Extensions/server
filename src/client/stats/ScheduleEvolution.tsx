import {
    BarController,
    BarElement,
    CategoryScale,
    Chart,
    Legend,
    LinearScale,
    Tooltip,
} from 'chart.js';
import type { PublishTimeYear } from 'models/PublishTimeYear';
import type { ScheduleEvolutionYear } from 'models/ScheduleEvolutionYear';
import { useEffect, useRef, useState } from 'react';

import { getStatsPublishTimeEvolution } from 'bindings/api/GetStatsPublishTimeEvolution';
import { getStatsScheduleEvolution } from 'bindings/api/GetStatsScheduleEvolution';

Chart.register(
    BarController,
    BarElement,
    CategoryScale,
    LinearScale,
    Tooltip,
    Legend,
);

const DOW_LABELS = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'];
const DOW_COLORS = [
    'rgba(239, 68, 68, 0.8)',
    'rgba(251, 146, 60, 0.8)',
    'rgba(251, 191, 36, 0.8)',
    'rgba(34, 197, 94, 0.8)',
    'rgba(99, 102, 241, 0.8)',
    'rgba(168, 85, 247, 0.8)',
    'rgba(236, 72, 153, 0.8)',
];

interface ScheduleChartProps {
    data: ScheduleEvolutionYear[];
}

function ScheduleChart({ data }: ScheduleChartProps) {
    const canvasRef = useRef<HTMLCanvasElement>(null);
    const chartRef = useRef<Chart | null>(null);

    useEffect(() => {
        if (!canvasRef.current) return;
        chartRef.current?.destroy();

        chartRef.current = new Chart(canvasRef.current, {
            type: 'bar',
            data: {
                labels: data.map((d) => String(d.year)),
                datasets: DOW_LABELS.map((label, i) => ({
                    label,
                    data: data.map((d) => d.dowCounts[i] ?? 0),
                    backgroundColor: DOW_COLORS[i],
                    stack: 'a',
                })),
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
                                `${ctx.dataset.label ?? ''}: ${(ctx.parsed.y as number).toLocaleString()} comics`,
                        },
                    },
                },
                scales: {
                    x: { stacked: true },
                    y: {
                        stacked: true,
                        beginAtZero: true,
                        title: { display: true, text: 'Comics' },
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

interface PublishTimeHeatmapProps {
    data: PublishTimeYear[];
}

function PublishTimeHeatmap({ data }: PublishTimeHeatmapProps) {
    const canvasRef = useRef<HTMLCanvasElement>(null);

    useEffect(() => {
        const canvas = canvasRef.current;
        if (!canvas) return;
        const ctx = canvas.getContext('2d');
        if (!ctx) return;

        const LABEL_W = 44;
        const LABEL_H = 28;
        const CELL_W = Math.max(
            14,
            Math.floor((canvas.width - LABEL_W) / data.length),
        );
        const CELL_H = 18;
        const HOURS = 24;

        canvas.height = LABEL_H + HOURS * CELL_H + 8;

        const yearTotals = data.map((d) =>
            d.hourCounts.reduce((s, c) => s + c, 0),
        );

        ctx.clearRect(0, 0, canvas.width, canvas.height);
        ctx.font = '11px system-ui, sans-serif';
        ctx.textBaseline = 'middle';

        // Year labels along the top
        ctx.fillStyle = '#6b7280';
        ctx.textAlign = 'center';
        for (let xi = 0; xi < data.length; xi++) {
            const x = LABEL_W + xi * CELL_W + CELL_W / 2;
            ctx.fillText(String(data[xi]?.year ?? ''), x, LABEL_H / 2);
        }

        // Hour labels down the left
        ctx.textAlign = 'right';
        for (let h = 0; h < HOURS; h++) {
            const y = LABEL_H + h * CELL_H + CELL_H / 2;
            const label = String(h).padStart(2, '0');
            ctx.fillText(label, LABEL_W - 4, y);
        }

        // Cells
        for (let xi = 0; xi < data.length; xi++) {
            const total = yearTotals[xi] ?? 0;
            for (let h = 0; h < HOURS; h++) {
                const count = data[xi]?.hourCounts[h] ?? 0;
                const pct = total > 0 ? count / total : 0;
                // Map 0→white, 1→indigo-700 via a perceptual curve
                const t = Math.sqrt(pct);
                const r = Math.round(255 - t * (255 - 67));
                const g = Math.round(255 - t * (255 - 56));
                const b = Math.round(255 - t * (255 - 202));
                ctx.fillStyle = `rgb(${String(r)},${String(g)},${String(b)})`;
                ctx.fillRect(
                    LABEL_W + xi * CELL_W,
                    LABEL_H + h * CELL_H,
                    CELL_W - 1,
                    CELL_H - 1,
                );
            }
        }
    }, [data]);

    return (
        <div className="overflow-x-auto">
            <canvas
                ref={canvasRef}
                width={900}
                height={500}
                style={{ display: 'block', maxWidth: '100%' }}
            />
        </div>
    );
}

export default function ScheduleEvolution() {
    const [dowData, setDowData] = useState<ScheduleEvolutionYear[] | null>(
        null,
    );
    const [timeData, setTimeData] = useState<PublishTimeYear[] | null>(null);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        getStatsScheduleEvolution()
            .then(setDowData)
            .catch((e: unknown) =>
                setError(e instanceof Error ? e.message : String(e)),
            );

        getStatsPublishTimeEvolution()
            .then(setTimeData)
            .catch((e: unknown) =>
                setError(e instanceof Error ? e.message : String(e)),
            );
    }, []);

    if (error)
        return <p className="text-red-600">Failed to load data: {error}</p>;

    return (
        <div>
            <h2 className="text-xl font-semibold text-gray-800 mb-1">
                Schedule Evolution
            </h2>
            <p className="text-sm text-gray-500 mb-4">
                Comics published per day of week, broken down by year. Shows how
                the publishing schedule has shifted over the comic&apos;s
                lifetime — from irregular to regular, or between different
                weekday patterns.
            </p>
            {!dowData ? (
                <p className="text-gray-500">Loading…</p>
            ) : (
                <ScheduleChart data={dowData} />
            )}

            <h3 className="text-lg font-semibold text-gray-800 mt-8 mb-1">
                Publish Time of Day
            </h3>
            <p className="text-sm text-gray-500 mb-4">
                Distribution of publish times (UTC) by year, shown as a fraction
                of each year&apos;s total comics. Darker cells mean a higher
                share of that year&apos;s comics were published in that hour.
            </p>
            {!timeData ? (
                <p className="text-gray-500">Loading…</p>
            ) : (
                <PublishTimeHeatmap data={timeData} />
            )}
        </div>
    );
}
