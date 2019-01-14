using Microsoft.EntityFrameworkCore;
using Microsoft.EntityFrameworkCore.Metadata.Builders;
using QCExtensions.Domain.Entities;

namespace QCExtensions.Persistence.Configurations
{
	public class ComicConfiguration : IEntityTypeConfiguration<Comic>
	{
		public void Configure(EntityTypeBuilder<Comic> comicBuilder)
		{
			comicBuilder.ToTable("comic");
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
	}
}
