using Microsoft.EntityFrameworkCore;
using QCExtensions.Domain.Entities;

namespace QCExtensions.Server.Models
{
	public partial class ApplicationDbContext
	{
		protected override void OnModelCreating(ModelBuilder builder)
		{
			base.OnModelCreating(builder);

			BuildComicEntity(builder);
			BuildItemEntity(builder);
			BuildItemImageEntity(builder);
			BuildLogEntryEntity(builder);
			BuildNewsEntity(builder);
			BuildOccurrenceEntity(builder);
			BuildTokenEntity(builder);

			builder.Query<ComicEditorData>();
			builder.Query<ComicItemNavigationData>();
		}

		private static void BuildComicEntity(ModelBuilder builder)
		{
			var comicBuilder = builder.Entity<Comic>()
							.ToTable("comic");
			comicBuilder.HasKey(c => c.Id);
			comicBuilder.Property(c => c.Id)
				.HasColumnName("id");
			comicBuilder.Property(c => c.IsGuestComic)
				.HasColumnName("isGuestComic")
				.IsRequired();
			comicBuilder.Property(c => c.IsNonCanon)
				.HasColumnName("isNonCanon")
				.IsRequired();
			comicBuilder.Property(c => c.HasNoCast)
				.IsRequired();
			comicBuilder.Property(c => c.HasNoLocation)
				.IsRequired();
			comicBuilder.Property(c => c.HasNoStoryline)
				.IsRequired();
			comicBuilder.Property(c => c.HasNoTitle)
				.IsRequired();
			comicBuilder.Property(c => c.HasNoTagline)
				.IsRequired();
			comicBuilder.Property(c => c.Title)
				.HasColumnName("title")
				.IsRequired()
				.HasMaxLength(255);
			comicBuilder.Property(c => c.Tagline)
				.HasColumnName("tagline")
				.HasMaxLength(255);
			comicBuilder.Property(c => c.PublishDate)
				.HasColumnName("publishDate");
			comicBuilder.Property(c => c.IsAccuratePublishDate)
				.HasColumnName("isAccuratePublishDate")
				.IsRequired();
			comicBuilder.HasOne(c => c.News)
				.WithOne(n => n.Comic)
				.HasForeignKey<Comic>(c => c.Id);
		}

		private static void BuildItemEntity(ModelBuilder builder)
		{
			var itemBuilder = builder.Entity<Item>()
				.ToTable("items");
			itemBuilder.HasKey(i => i.Id);
			itemBuilder.Property(i => i.Id)
				.HasColumnName("id");
			itemBuilder.Property(i => i.ShortName)
				.HasColumnName("shortName")
				.HasMaxLength(50)
				.IsRequired();
			itemBuilder.Property(i => i.Name)
				.HasColumnName("name")
				.HasMaxLength(255)
				.IsRequired();
			itemBuilder.Property(i => i.Type)
				.HasColumnName("type")
				.HasMaxLength(255)
				.IsRequired();
			itemBuilder.Ignore(i => i.TypeValue);
			itemBuilder.OwnsOne(i => i.Color);
		}

		private static void BuildItemImageEntity(ModelBuilder builder)
		{
			var itemImageBuilder = builder.Entity<ItemImage>();
			itemImageBuilder.HasKey(i => i.Id);
			itemImageBuilder.Property(i => i.Image)
				.IsRequired();
			itemImageBuilder.Property(i => i.CRC32CHash)
				.IsRequired();
		}

		private static void BuildLogEntryEntity(ModelBuilder builder)
		{
			var logEntryBuilder = builder.Entity<LogEntry>();
			logEntryBuilder.ToTable("log_entry");
			logEntryBuilder.HasKey(l => l.Id);
			logEntryBuilder.Property(l => l.Id)
				.HasColumnName("id");
			logEntryBuilder.HasOne(l => l.Token)
				.WithMany(t => t.LogEntries)
				.HasForeignKey(l => l.UserToken);
			logEntryBuilder.Property(l => l.DateTime)
				.IsRequired();
			logEntryBuilder.Property(l => l.Action)
				.IsRequired();
		}

		private static void BuildNewsEntity(ModelBuilder builder)
		{
			var newsBuilder = builder.Entity<News>();
			newsBuilder.ToTable("news");
			newsBuilder.HasKey(n => n.ComicId);
			newsBuilder.Property(n => n.ComicId)
				.HasColumnName("comic");
			newsBuilder.Property(n => n.LastUpdated)
				.HasColumnName("lastUpdated")
				.IsRequired();
			newsBuilder.Property(n => n.NewsText)
				.HasColumnName("news")
				.IsRequired();
			newsBuilder.Property(n => n.UpdateFactor)
				.HasColumnName("updateFactor")
				.IsRequired();
			newsBuilder.Property(n => n.IsLocked)
				.HasColumnName("isLocked")
				.IsRequired();
			newsBuilder.Ignore(n => n.IsOutdated);
		}

		private static void BuildOccurrenceEntity(ModelBuilder builder)
		{
			var occurenceBuilder = builder.Entity<Occurrence>();
			occurenceBuilder.ToTable("occurences");
			occurenceBuilder.HasKey(o => new { o.ComicId, o.ItemId });
			occurenceBuilder.Property(o => o.ComicId)
				.HasColumnName("comic_id");
			occurenceBuilder.Property(o => o.ItemId)
				.HasColumnName("items_id");
		}

		private static void BuildTokenEntity(ModelBuilder builder)
		{
			var tokenBuilder = builder.Entity<Token>();
			tokenBuilder.ToTable("token");
			tokenBuilder.HasKey(t => t.Id);
			tokenBuilder.Property(t => t.Id)
				.HasColumnName("id");
			tokenBuilder.Property(t => t.Identifier)
				.HasMaxLength(50)
				.IsRequired();
		}
	}
}
