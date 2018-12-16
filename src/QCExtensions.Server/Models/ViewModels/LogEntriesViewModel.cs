using System;
using Newtonsoft.Json;

namespace QCExtensions.Server.Models.ViewModels
{
	public class LogEntryViewModel
	{
		public string Identifier { get; set; }
		public DateTime DateTime { get; set; }
		public string Action { get; set; }
	}

	public class LogEntriesViewModel
	{
		public LogEntryViewModel[] LogEntries { get; set; }
		public int Page { get; set; }
		public int PageCount { get; set; }
		public int LogEntryCount { get; set; }
	}
}