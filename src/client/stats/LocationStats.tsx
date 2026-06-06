import ItemStatsTable from './ItemStatsTable';

export default function LocationStats() {
    return (
        <ItemStatsTable
            endpoint="/api/v3/stats/locations"
            title="Location Stats"
            description="All locations ranked by total number of comic appearances."
            sortBy="appearances"
        />
    );
}
