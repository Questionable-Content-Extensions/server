

using System.Linq;
using System.Threading.Tasks;
using Microsoft.EntityFrameworkCore;
using QCExtensions.Domain.Entities;

namespace QCExtensions.Application.Extensions.DbContext
{
	public static class ItemExtensions
	{
		public static async Task<Item> GetByIdAsync(this DbSet<Item> items, int id,
			bool includeOccurrences = false, bool includeComics = false,
			bool includeImages = false)
		{
			IQueryable<Item> query = items;
			if (includeOccurrences || includeComics)
			{
				var iquery = query.Include(i => i.Occurrences);
				if (includeComics)
				{
					query = iquery.ThenInclude(o => o.Comic);
				}
				else
				{
					query = iquery;
				}
			}
			if (includeImages)
			{
				query = query.Include(i => i.Images);
			}

			return await query.SingleOrDefaultAsync(item => item.Id == id);
		}

		public static async Task<bool> ExistsAsync(this DbSet<Item> items, int id)
		{
			return await items.AnyAsync(o => o.Id == id);
		}
	}
}