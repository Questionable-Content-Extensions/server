import type { ItemStats } from 'models/ItemStats';

import ItemStatsTable from './ItemStatsTable';

interface LocationDebutsProps {
    sharedData?: ItemStats[] | null;
    sharedError?: string | null;
}

export default function LocationDebuts({
    sharedData,
    sharedError,
}: LocationDebutsProps) {
    return (
        <ItemStatsTable
            endpoint="/api/v3/stats/locations"
            title="Location Debuts"
            description="All locations in order of first appearance."
            sortBy="firstComic"
            sharedData={sharedData}
            sharedError={sharedError}
        />
    );
}
