import {
    BarController,
    BarElement,
    CategoryScale,
    Chart,
    LinearScale,
    Tooltip,
} from 'chart.js';
import { useEffect, useLayoutEffect, useRef, useState } from 'react';

import type { CharacterMeta } from '../../../bindings/CharacterMeta';
import type { YearlySpotlightResponse } from '../../../bindings/YearlySpotlightResponse';
import { contrastColor } from './colorUtils';

Chart.register(BarController, BarElement, CategoryScale, LinearScale, Tooltip);

function buildDatasets(response: YearlySpotlightResponse) {
    return Array.from({ length: 5 }, (_, rank) => ({
        label: `Rank ${rank + 1}`,
        backgroundColor: response.years.map((year) => {
            const entry = year.characters[rank];
            if (!entry) return 'rgba(0,0,0,0)';
            const meta = response.characters[entry.id];
            return meta ? contrastColor(meta.color) : '#ccc';
        }),
        data: response.years.map(
            (year) => year.characters[rank]?.appearances ?? null,
        ),
    }));
}

interface LegendProps {
    characters: { [key: number]: CharacterMeta };
}

function CharLegend({ characters }: LegendProps) {
    return (
        <div className="flex flex-wrap gap-x-4 gap-y-1">
            {Object.values(characters).map((meta) => (
                <span
                    key={meta.name}
                    className="inline-flex items-center gap-1.5 text-xs text-gray-700"
                >
                    <span
                        className="inline-block w-3 h-3 rounded-sm shrink-0"
                        style={{ backgroundColor: contrastColor(meta.color) }}
                    />
                    {meta.name}
                </span>
            ))}
        </div>
    );
}

interface Props {
    response: YearlySpotlightResponse;
}

export default function YearlyBarChart({ response }: Props) {
    const canvasRef = useRef<HTMLCanvasElement>(null);
    const chartRef = useRef<Chart | null>(null);
    const legendRef = useRef<HTMLDivElement>(null);
    const [legendHeight, setLegendHeight] = useState(0);

    useLayoutEffect(() => {
        if (legendRef.current) {
            setLegendHeight(legendRef.current.offsetHeight);
        }
    }, [response]);

    useEffect(() => {
        if (!canvasRef.current) return;

        chartRef.current?.destroy();

        const nameAt = (yearIdx: number, rank: number) => {
            const entry = response.years[yearIdx]?.characters[rank];
            return entry ? (response.characters[entry.id]?.name ?? '') : '';
        };

        chartRef.current = new Chart(canvasRef.current, {
            type: 'bar',
            data: {
                labels: response.years.map((y) => String(y.year)),
                datasets: buildDatasets(response),
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                animation: false,
                plugins: {
                    legend: { display: false },
                    tooltip: {
                        callbacks: {
                            label: (ctx) => {
                                const name = nameAt(
                                    ctx.dataIndex,
                                    ctx.datasetIndex,
                                );
                                return name
                                    ? `${name}: ${(ctx.parsed.x as number).toLocaleString()} comics`
                                    : '';
                            },
                        },
                    },
                },
                indexAxis: 'y' as const,
                scales: {
                    x: {
                        beginAtZero: true,
                        title: { display: true, text: 'Comics appeared in' },
                    },
                    y: { stacked: false },
                },
            },
        });

        return () => {
            chartRef.current?.destroy();
            chartRef.current = null;
        };
    }, [response]);

    return (
        <>
            <div style={{ paddingBottom: legendHeight }}>
                <div
                    className="relative"
                    style={{ height: `${response.years.length * 60}px` }}
                >
                    <canvas ref={canvasRef} />
                </div>
            </div>
            <div
                ref={legendRef}
                className="fixed bottom-0 left-0 right-0 z-10 bg-white border-t border-gray-200 px-6 py-3 max-w-7xl mx-auto"
            >
                <CharLegend characters={response.characters} />
            </div>
        </>
    );
}
