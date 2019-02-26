using QCExtensions.Application.Logs.Models;

namespace QCExtensions.Application.Logs.Queries.GetLogs
{
	public class GetLogsQuery : RequestWithToken<LogEntriesDto>
	{
		public int Page { get; set; }
		public int PageSize { get; set; }
	}
}
