namespace QCExtensions.Application.Logs.Models
{
	public class LogEntriesDto
	{
		public LogDto[] LogEntries { get; set; }
		public int Page { get; set; }
		public int PageCount { get; set; }
		public int LogEntryCount { get; set; }
	}
}
