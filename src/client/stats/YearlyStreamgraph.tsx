import {
    type SeriesPoint,
    area,
    curveBasis,
    stack,
    stackOffsetWiggle,
    stackOrderInsideOut,
} from 'd3-shape';
import { useEffect, useMemo, useRef, useState } from 'react';

import type { CharacterMeta } from '../../../bindings/CharacterMeta';
import type { YearlySpotlightResponse } from '../../../bindings/YearlySpotlightResponse';
import { contrastColor } from './colorUtils';

interface StreamRow {
    year: number;
    [key: string]: number;
}

const MARGIN = { top: 10, right: 10, bottom: 30, left: 10 };
const HEIGHT = 480;

interface TooltipState {
    x: number;
    y: number;
    name: string;
    year: number;
    count: number;
}

interface LegendProps {
    characters: Record<number, CharacterMeta>;
}

function StreamLegend({ characters }: LegendProps) {
    return (
        <div className="mt-4 flex flex-wrap gap-x-4 gap-y-1">
            {Object.values(characters).map((meta) => (
                <span
                    key={meta.name}
                    className="inline-flex items-center gap-1.5 text-xs text-gray-700"
                >
                    <span
                        className="inline-block h-3 w-3 shrink-0 rounded-sm"
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

export default function YearlyStreamgraph({ response }: Props) {
    const containerRef = useRef<HTMLDivElement>(null);
    const [width, setWidth] = useState(800);
    const [hoveredId, setHoveredId] = useState<string | null>(null);
    const [tooltip, setTooltip] = useState<TooltipState | null>(null);

    useEffect(() => {
        const el = containerRef.current;
        if (!el) return;
        const ro = new ResizeObserver((entries) => {
            const w = entries[0]?.contentRect.width;
            if (w && w > 0) setWidth(w);
        });
        ro.observe(el);
        return () => {
            ro.disconnect();
        };
    }, []);

    const innerWidth = width - MARGIN.left - MARGIN.right;
    const innerHeight = HEIGHT - MARGIN.top - MARGIN.bottom;

    const { years, layers, yMin, yMax, appearanceMap } = useMemo(() => {
        const charIds = Object.keys(response.characters);
        const years = response.years.map((y) => y.year);

        const appearanceMap = new Map<number, Map<number, number>>();
        const stackData: StreamRow[] = response.years.map((yearEntry, idx) => {
            const byChar = new Map<number, number>();
            const row: StreamRow = { year: yearEntry.year };
            for (const c of yearEntry.characters) {
                byChar.set(c.id, c.appearances);
            }
            appearanceMap.set(idx, byChar);
            for (const id of charIds) {
                row[id] = byChar.get(Number(id)) ?? 0;
            }
            return row;
        });

        const layers = stack<StreamRow, string>()
            .keys(charIds)
            .offset(stackOffsetWiggle)
            .order(stackOrderInsideOut)(stackData);

        let yMin = Infinity;
        let yMax = -Infinity;
        for (const layer of layers) {
            for (const pt of layer) {
                if (pt[0] < yMin) yMin = pt[0];
                if (pt[1] > yMax) yMax = pt[1];
            }
        }

        return { years, layers, yMin, yMax, appearanceMap };
    }, [response]);

    const xStep =
        years.length > 1 ? innerWidth / (years.length - 1) : innerWidth;

    const areaGen = useMemo(() => {
        const yRange = yMax - yMin || 1;
        const yPos = (v: number) =>
            MARGIN.top + innerHeight * (1 - (v - yMin) / yRange);
        return area<SeriesPoint<StreamRow>>()
            .x((_, i) => MARGIN.left + i * xStep)
            .y0((d) => yPos(d[0]))
            .y1((d) => yPos(d[1]))
            .curve(curveBasis);
    }, [xStep, yMin, yMax, innerHeight]);

    function handleMouseMove(
        e: React.MouseEvent<SVGPathElement>,
        charId: string,
    ) {
        const svgEl = (e.currentTarget as Element).closest('svg')!;
        const rect = svgEl.getBoundingClientRect();
        const mouseX = e.clientX - rect.left - MARGIN.left;
        const yearIdx = Math.max(
            0,
            Math.min(years.length - 1, Math.round(mouseX / xStep)),
        );
        const count = appearanceMap.get(yearIdx)?.get(Number(charId)) ?? 0;
        const name = response.characters[Number(charId)]?.name ?? '';
        setTooltip({
            x: e.clientX - rect.left,
            y: e.clientY - rect.top,
            name,
            year: years[yearIdx],
            count,
        });
    }

    const labelEvery = Math.max(1, Math.ceil(years.length / 12));

    return (
        <>
            <div className="relative" ref={containerRef}>
                <svg width={width} height={HEIGHT}>
                    {layers.map((layer) => {
                        const charId = layer.key;
                        const meta = response.characters[Number(charId)];
                        const color = meta ? contrastColor(meta.color) : '#ccc';
                        const isHovered = hoveredId === charId;
                        const opacity =
                            hoveredId === null ? 0.85 : isHovered ? 1 : 0.25;
                        return (
                            <path
                                key={charId}
                                d={areaGen(layer) ?? undefined}
                                fill={color}
                                fillOpacity={opacity}
                                stroke="white"
                                strokeWidth={0.5}
                                style={{
                                    cursor: 'crosshair',
                                    transition: 'fill-opacity 0.15s ease',
                                }}
                                onMouseMove={(e) => {
                                    setHoveredId(charId);
                                    handleMouseMove(e, charId);
                                }}
                                onMouseLeave={() => {
                                    setHoveredId(null);
                                    setTooltip(null);
                                }}
                            />
                        );
                    })}
                    {years.map((year, i) => {
                        if (i % labelEvery !== 0 && i !== years.length - 1)
                            return null;
                        return (
                            <text
                                key={year}
                                x={MARGIN.left + i * xStep}
                                y={HEIGHT - 6}
                                textAnchor="middle"
                                fontSize="11"
                                fill="#6b7280"
                            >
                                {year}
                            </text>
                        );
                    })}
                </svg>
                {tooltip && (
                    <div
                        data-testid="streamgraph-tooltip"
                        className="pointer-events-none absolute z-20 rounded border border-gray-200 bg-white px-3 py-1.5 text-xs text-gray-700 shadow-sm"
                        style={{ left: tooltip.x + 12, top: tooltip.y - 36 }}
                    >
                        <span className="font-medium">{tooltip.name}</span>
                        {' — '}
                        {tooltip.year}:{' '}
                        {tooltip.count > 0
                            ? `${tooltip.count.toLocaleString()} comics`
                            : 'not in top 5'}
                    </div>
                )}
            </div>
            <StreamLegend characters={response.characters} />
        </>
    );
}
