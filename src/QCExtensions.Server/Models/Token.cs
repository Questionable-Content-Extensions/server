using System;
using System.Collections.Generic;
using System.ComponentModel.DataAnnotations;
using System.ComponentModel.DataAnnotations.Schema;
using Microsoft.EntityFrameworkCore;

namespace QCExtensions.Server.Models
{
	[Table("token")]
	public class Token
	{
		[Key]
		[Column("id")]
		public Guid Id { get; set; }

		public string Identifier { get; set; }

		[InverseProperty("Token")]
		public ICollection<LogEntry> LogEntries { get; set; }
	}
}