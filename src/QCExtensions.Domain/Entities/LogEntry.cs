using System;
using System.Collections.Generic;

namespace QCExtensions.Domain.Entities
{
	public class LogEntry
	{
		public int Id { get; set; }
		public Guid UserToken { get; set; }
		public Token Token { get; set; }
		public DateTime DateTime { get; set; }
		public string Action { get; set; }
	}
}