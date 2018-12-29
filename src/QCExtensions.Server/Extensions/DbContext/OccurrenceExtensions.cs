using System.Linq;
using System.Threading.Tasks;
using Microsoft.EntityFrameworkCore;
using QCExtensions.Domain.Entities;

namespace QCExtensions.Server.Extensions.DbContext
{
	public static class OccurenceExtensions
	{
		public static async Task<Occurrence> GetByComicIdAndItemIdAsync(this DbSet<Occurrence> occurrences, int comicId, int itemId, bool includeComic = false, bool includeItem = false)
		{
			IQueryable<Occurrence> query = occurrences;
			if (includeComic)
			{
				query = query.Include(o => o.Comic);
			}
			if (includeItem)
			{
				query = query.Include(o => o.Item);
			}

			return await query.SingleOrDefaultAsync(o => o.ComicId == comicId && o.ItemId == itemId);
		}

		public static async Task<bool> ExistsAsync(this DbSet<Occurrence> occurrences, int comicId, int itemId)
		{
			return await occurrences.AnyAsync(o => o.ComicId == comicId && o.ItemId == itemId);
		}
	}
}