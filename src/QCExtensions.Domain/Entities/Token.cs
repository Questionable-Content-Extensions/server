using System;
using System.Collections.Generic;

namespace QCExtensions.Domain.Entities
{
	public class Token
	{
		public Token()
		{
			LogEntries = new HashSet<LogEntry>();
		}
		public Guid Id { get; set; }
		public string Identifier { get; set; }
		public ICollection<LogEntry> LogEntries { get; private set; }
	}
}