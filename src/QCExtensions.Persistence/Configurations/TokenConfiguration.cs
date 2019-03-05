using Microsoft.EntityFrameworkCore;
using Microsoft.EntityFrameworkCore.Metadata.Builders;
using QCExtensions.Domain.Entities;

namespace QCExtensions.Persistence.Configurations
{
	public class TokenConfiguration : IEntityTypeConfiguration<Token>
	{
		public void Configure(EntityTypeBuilder<Token> tokenBuilder)
		{
			tokenBuilder.ToTable("token");
			tokenBuilder.HasKey(t => t.Id);
			tokenBuilder.Property(t => t.Id)
				.HasColumnName("id");
			tokenBuilder.Property(t => t.Identifier)
				.HasMaxLength(50)
				.IsRequired();

			tokenBuilder.Property(t => t.CanAddImageToItem)
				.IsRequired();
			tokenBuilder.Property(t => t.CanAddItemToComic)
				.IsRequired();
			tokenBuilder.Property(t => t.CanChangeComicData)
				.IsRequired();
			tokenBuilder.Property(t => t.CanChangeItemData)
				.IsRequired();
			tokenBuilder.Property(t => t.CanRemoveImageFromItem)
				.IsRequired();
			tokenBuilder.Property(t => t.CanRemoveItemFromComic)
				.IsRequired();
		}
	}
}
