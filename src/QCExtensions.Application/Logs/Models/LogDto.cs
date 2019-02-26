using System;

namespace QCExtensions.Application.Logs.Models
{

	public class LogDto
	{
		public string Identifier { get; set; }
		public DateTime DateTime { get; set; }
		public string Action { get; set; }
	}
}
