import type { ItemStats } from 'models/ItemStats';

import ItemStatsTable from './ItemStatsTable';

interface LocationStatsProps {
    sharedData?: ItemStats[] | null;
    sharedError?: string | null;
}

export default function LocationStats({
    sharedData,
    sharedError,
}: LocationStatsProps) {
    return (
        <ItemStatsTable
            endpoint="/api/v3/stats/locations"
            title="Location Stats"
            description="All locations ranked by total number of comic appearances."
            sortBy="appearances"
            sharedData={sharedData}
            sharedError={sharedError}
        />
    );
}
