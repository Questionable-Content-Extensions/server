import { useState } from 'react';

export type SortDir = 'asc' | 'desc';

export interface SortState<TKey extends string> {
    key: TKey;
    dir: SortDir;
}

export function useSortState<TKey extends string>(
    defaultKey: TKey,
    defaultDir: SortDir = 'desc',
): [SortState<TKey>, (key: TKey) => void] {
    const [sort, setSort] = useState<SortState<TKey>>({
        key: defaultKey,
        dir: defaultDir,
    });

    function handleSort(key: TKey) {
        setSort((prev) => ({
            key,
            dir:
                prev.key === key
                    ? prev.dir === 'asc'
                        ? 'desc'
                        : 'asc'
                    : defaultDir,
        }));
    }

    return [sort, handleSort];
}

interface SortableHeaderProps<TKey extends string> {
    sortKey: TKey;
    sort: SortState<TKey>;
    onSort: (key: TKey) => void;
    align?: 'left' | 'right';
    children: React.ReactNode;
}

export function SortableHeader<TKey extends string>({
    sortKey,
    sort,
    onSort,
    align = 'right',
    children,
}: SortableHeaderProps<TKey>): React.JSX.Element {
    const isActive = sort.key === sortKey;
    const arrow = isActive ? (sort.dir === 'asc' ? ' ↑' : ' ↓') : '';
    const classes = [
        'py-2 pr-4 font-medium cursor-pointer select-none hover:text-gray-900',
        align === 'right' ? 'text-right' : 'text-left',
        isActive ? 'text-gray-900' : '',
    ]
        .filter(Boolean)
        .join(' ');
    return (
        <th
            className={classes}
            onClick={() => {
                onSort(sortKey);
            }}
        >
            {children}
            {arrow}
        </th>
    );
}

interface StaticHeaderProps {
    align?: 'left' | 'right';
    className?: string;
    children: React.ReactNode;
}

export function StaticHeader({
    align,
    className,
    children,
}: StaticHeaderProps): React.JSX.Element {
    const classes = ['py-2 pr-4 font-medium'];
    if (align === 'right') classes.push('text-right');
    else if (align === 'left') classes.push('text-left');
    if (className) classes.push(className);
    return <th className={classes.join(' ')}>{children}</th>;
}

export function StatsTable({
    children,
}: {
    children: React.ReactNode;
}): React.JSX.Element {
    return (
        <div className="overflow-x-auto">
            <table className="min-w-full text-sm">{children}</table>
        </div>
    );
}

export function StatsTheadRow({
    children,
}: {
    children: React.ReactNode;
}): React.JSX.Element {
    return (
        <tr className="border-b border-gray-200 text-left text-gray-600">
            {children}
        </tr>
    );
}

export function StatsTbodyRow({
    children,
}: {
    children: React.ReactNode;
}): React.JSX.Element {
    return (
        <tr className="border-b border-gray-100 hover:bg-gray-50">
            {children}
        </tr>
    );
}

export function comicLink(comicId: number): string {
    return `https://questionablecontent.net/view.php?comic=${comicId}`;
}
