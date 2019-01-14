using Microsoft.EntityFrameworkCore;
using Microsoft.EntityFrameworkCore.Metadata.Builders;
using QCExtensions.Domain.Entities;

namespace QCExtensions.Persistence.Configurations
{
	public class OccurrenceConfiguration : IEntityTypeConfiguration<Occurrence>
	{
		public void Configure(EntityTypeBuilder<Occurrence> occurrenceBuilder)
		{
			occurrenceBuilder.ToTable("occurences");
			occurrenceBuilder.HasKey(o => new { o.ComicId, o.ItemId });
			occurrenceBuilder.Property(o => o.ComicId)
				.HasColumnName("comic_id");
			occurrenceBuilder.Property(o => o.ItemId)
				.HasColumnName("items_id");
		}
	}
}
