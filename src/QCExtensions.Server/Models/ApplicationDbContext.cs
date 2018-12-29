using Microsoft.EntityFrameworkCore;
using QCExtensions.Domain.Entities;

namespace QCExtensions.Server.Models
{
	public partial class ApplicationDbContext : DbContext
	{
		public ApplicationDbContext(DbContextOptions<ApplicationDbContext> options)
			: base(options)
		{
		}

		public DbSet<News> News { get; set; }
		public DbSet<Comic> Comics { get; set; }
		public DbSet<Item> Items { get; set; }
		public DbSet<ItemImage> ItemImages { get; set; }
		public DbSet<Occurrence> Occurrences { get; set; }
		public DbSet<Token> Tokens { get; set; }
		public DbSet<LogEntry> LogEntries { get; set; }
	}
}
