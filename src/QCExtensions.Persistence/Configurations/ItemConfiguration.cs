using Microsoft.EntityFrameworkCore;
using Microsoft.EntityFrameworkCore.Metadata.Builders;
using QCExtensions.Domain.Entities;

namespace QCExtensions.Persistence.Configurations
{
	public class ItemConfiguration : IEntityTypeConfiguration<Item>
	{
		public void Configure(EntityTypeBuilder<Item> itemBuilder)
		{
			itemBuilder.ToTable("items");
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
	}
}
