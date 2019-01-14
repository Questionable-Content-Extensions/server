using System.Linq;
using Microsoft.EntityFrameworkCore;

namespace QCExtensions.Domain.Entities
{
	public abstract class DomainDbContext : DbContext
	{
		public DomainDbContext(DbContextOptions options) : base(options) { }
		protected DomainDbContext() { }

		public DbSet<News> News { get; set; }
		public DbSet<Comic> Comics { get; set; }
		public DbSet<Item> Items { get; set; }
		public DbSet<ItemImage> ItemImages { get; set; }
		public DbSet<Occurrence> Occurrences { get; set; }
		public DbSet<Token> Tokens { get; set; }
		public DbSet<LogEntry> LogEntries { get; set; }

		public abstract IQueryable<ComicEditorData> QueryComicEditorData(int comicId);
		public abstract IQueryable<ComicItemNavigationData> QueryComicItemNavigationData(int comicId, string exclude = null);
		public abstract IQueryable<ComicItemNavigationData> QueryComicAllItemNavigationData(int comicId, string exclude = null);
	}
}
