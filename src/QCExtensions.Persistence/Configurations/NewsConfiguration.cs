using Microsoft.EntityFrameworkCore;
using Microsoft.EntityFrameworkCore.Metadata.Builders;
using QCExtensions.Domain.Entities;

namespace QCExtensions.Persistence.Configurations
{
	public class NewsConfiguration : IEntityTypeConfiguration<News>
	{
		public void Configure(EntityTypeBuilder<News> newsBuilder)
		{
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
	}
}
