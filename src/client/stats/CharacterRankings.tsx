import ItemStatsTable from './ItemStatsTable';

export default function CharacterRankings() {
    return (
        <ItemStatsTable
            endpoint="/api/v3/stats/cast"
            title="Character Rankings"
            description="All cast members ranked by total number of comic appearances."
            sortBy="appearances"
        />
    );
}
