using Microsoft.AspNetCore.Identity.EntityFrameworkCore;
using Microsoft.EntityFrameworkCore;
using System.ComponentModel.DataAnnotations;

namespace QCExtensions.Server.Models
{
	public class ApplicationDbContext : DbContext
	{
		public ApplicationDbContext(DbContextOptions<ApplicationDbContext> options)
			: base(options)
		{
		}

		protected override void OnModelCreating(ModelBuilder builder)
		{
			base.OnModelCreating(builder);

			builder.Entity<Occurrence>()
				.HasKey(o => new { o.ComicId, o.ItemId });
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
