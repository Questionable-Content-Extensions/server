using Microsoft.EntityFrameworkCore;
using Microsoft.EntityFrameworkCore.Metadata.Builders;
using QCExtensions.Domain.Entities;

namespace QCExtensions.Persistence.Configurations
{
	public class ItemImageConfiguration : IEntityTypeConfiguration<ItemImage>
	{
		public void Configure(EntityTypeBuilder<ItemImage> itemImageBuilder)
		{
			itemImageBuilder.HasKey(i => i.Id);
			itemImageBuilder.Property(i => i.Image)
				.IsRequired();
			itemImageBuilder.Property(i => i.CRC32CHash)
				.IsRequired();
		}
	}
}
