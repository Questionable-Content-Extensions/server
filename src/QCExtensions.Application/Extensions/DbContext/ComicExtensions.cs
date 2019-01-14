using System.Linq;
using System.Threading.Tasks;
using Microsoft.EntityFrameworkCore;
using QCExtensions.Domain.Entities;

namespace QCExtensions.Application.Extensions.DbContext
{
	public static class ComicExtensions
	{
		public static async Task<Comic> GetByIdAsync(this DbSet<Comic> comics, int id, bool includeNews = false, bool includeOccurrences = false, bool includeItems = false)
		{
			IQueryable<Comic> query = comics;
			if (includeNews)
			{
				query = query.Include(c => c.News);
			}
			if (includeOccurrences || includeItems)
			{
				var iquery = query.Include(c => c.Occurrences);
				if (includeItems)
				{
					query = iquery.ThenInclude(o => o.Item);
				}
				else
				{
					query = iquery;
				}
			}

			return await query.SingleOrDefaultAsync(c => c.Id == id);
		}

		public static async Task<bool> ExistsAsync(this DbSet<Comic> comics, int id)
		{
			return await comics.AnyAsync(o => o.Id == id);
		}

		public static async Task<(Comic comic, bool wasCreated)> GetOrCreateAsync(this DbSet<Comic> comics, int comicId)
		{
			var wasCreated = false;
			var comic = await comics.GetByIdAsync(comicId);
			if (comic == null)
			{
				comic = new Comic { Id = comicId, Title = "" };
				comics.Add(comic);
				wasCreated = true;
			}
			return (comic, wasCreated);
		}
	}
}