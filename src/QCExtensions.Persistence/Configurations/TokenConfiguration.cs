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
		}
	}
}
