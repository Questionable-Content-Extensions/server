using System.Linq;
using Microsoft.EntityFrameworkCore;
using MySql.Data.MySqlClient;
using QCExtensions.Server.Models;

namespace QCExtensions.Server.Extensions.DbContext
{
	public static class QueryExtensions
	{
		public static IQueryable<ComicEditorData> QueryComicEditorData(this ApplicationDbContext applicationDbContext, int comicId)
		{
			return applicationDbContext.Query<ComicEditorData>().AsNoTracking().FromSql(
				"CALL `ComicEditorData`(@comicId)",
				new MySqlParameter("@comicId", comicId));
		}

		public static IQueryable<ComicItemNavigationData> QueryComicItemNavigationData(this ApplicationDbContext applicationDbContext, int comicId, string exclude = null)
		{
			return applicationDbContext.Query<ComicItemNavigationData>().AsNoTracking().FromSql(
				"CALL `ComicItemNavigationData`(@comicId, @exclude)",
				new MySqlParameter("@comicId", comicId),
				new MySqlParameter("@exclude", exclude));
		}

		public static IQueryable<ComicItemNavigationData> QueryComicAllItemNavigationData(this ApplicationDbContext applicationDbContext, int comicId, string exclude = null)
		{
			return applicationDbContext.Query<ComicItemNavigationData>().AsNoTracking().FromSql(
				"CALL `ComicAllItemNavigationData`(@comicId, @exclude)",
				new MySqlParameter("@comicId", comicId),
				new MySqlParameter("@exclude", exclude));
		}
	}
}