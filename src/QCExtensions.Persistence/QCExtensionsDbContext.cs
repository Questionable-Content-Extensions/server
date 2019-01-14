using Microsoft.EntityFrameworkCore;
using MySql.Data.MySqlClient;
using QCExtensions.Domain.Entities;
using System;
using System.Linq;

namespace QCExtensions.Persistence
{
	public class QCExtensionsDbContext : DomainDbContext
	{
		public QCExtensionsDbContext(DbContextOptions<QCExtensionsDbContext> options) : base(options) { }
		protected QCExtensionsDbContext() { }

		public override IQueryable<ComicEditorData> QueryComicEditorData(int comicId)
		{
			return Query<ComicEditorData>().AsNoTracking().FromSql(
				"CALL `ComicEditorData`(@comicId)",
				new MySqlParameter("@comicId", comicId));
		}
		public override IQueryable<ComicItemNavigationData> QueryComicItemNavigationData(int comicId, string exclude = null)
		{
			return Query<ComicItemNavigationData>().AsNoTracking().FromSql(
				"CALL `ComicItemNavigationData`(@comicId, @exclude)",
				new MySqlParameter("@comicId", comicId),
				new MySqlParameter("@exclude", exclude));
		}
		public override IQueryable<ComicItemNavigationData> QueryComicAllItemNavigationData(int comicId, string exclude = null)
		{
			return Query<ComicItemNavigationData>().AsNoTracking().FromSql(
				"CALL `ComicAllItemNavigationData`(@comicId, @exclude)",
				new MySqlParameter("@comicId", comicId),
				new MySqlParameter("@exclude", exclude));
		}

		protected override void OnModelCreating(ModelBuilder modelBuilder)
		{
			modelBuilder.Query<ComicEditorData>();
			modelBuilder.Query<ComicItemNavigationData>();

			modelBuilder.ApplyConfigurationsFromAssembly(typeof(QCExtensionsDbContext).Assembly);
		}
	}
}
