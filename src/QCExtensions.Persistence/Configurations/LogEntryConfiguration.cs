using Microsoft.EntityFrameworkCore;
using Microsoft.EntityFrameworkCore.Metadata.Builders;
using QCExtensions.Domain.Entities;

namespace QCExtensions.Persistence.Configurations
{
	public class LogEntryConfiguration : IEntityTypeConfiguration<LogEntry>
	{
		public void Configure(EntityTypeBuilder<LogEntry> logEntryBuilder)
		{
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
	}
}
