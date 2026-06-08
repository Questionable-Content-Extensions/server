import { act, cleanup, renderHook } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';

import { useSortState } from './StatsTable';

afterEach(cleanup);

describe('useSortState', () => {
    it('initialises with the given key and direction', () => {
        const { result } = renderHook(() =>
            useSortState<'name' | 'count'>('name', 'asc'),
        );
        const [sort] = result.current;
        expect(sort).toEqual({ key: 'name', dir: 'asc' });
    });

    it('defaults direction to desc when not specified', () => {
        const { result } = renderHook(() =>
            useSortState<'name' | 'count'>('count'),
        );
        const [sort] = result.current;
        expect(sort).toEqual({ key: 'count', dir: 'desc' });
    });

    it('toggles direction when the same key is clicked twice', () => {
        const { result } = renderHook(() =>
            useSortState<'name' | 'count'>('name', 'asc'),
        );
        const [, handleSort] = result.current;
        act(() => {
            handleSort('name');
        });
        expect(result.current[0]).toEqual({ key: 'name', dir: 'desc' });
        act(() => {
            handleSort('name');
        });
        expect(result.current[0]).toEqual({ key: 'name', dir: 'asc' });
    });

    it('switches to a new key using defaultDir', () => {
        const { result } = renderHook(() =>
            useSortState<'name' | 'count'>('name', 'desc'),
        );
        const [, handleSort] = result.current;
        act(() => {
            handleSort('count');
        });
        expect(result.current[0]).toEqual({ key: 'count', dir: 'desc' });
    });

    it('uses defaultDir (asc) when switching to a new key', () => {
        const { result } = renderHook(() =>
            useSortState<'name' | 'count'>('name', 'asc'),
        );
        const [, handleSort] = result.current;
        act(() => {
            handleSort('count');
        });
        expect(result.current[0]).toEqual({ key: 'count', dir: 'asc' });
    });

    it('does not change dir when switching back to initial key', () => {
        const { result } = renderHook(() =>
            useSortState<'name' | 'count'>('name', 'desc'),
        );
        const [, handleSort] = result.current;
        act(() => {
            handleSort('count');
        });
        act(() => {
            handleSort('name');
        });
        // switching back to 'name' resets to defaultDir, not the initial dir
        expect(result.current[0]).toEqual({ key: 'name', dir: 'desc' });
    });
});
