using System;
using System.Collections.Generic;
using System.ComponentModel.DataAnnotations;
using System.ComponentModel.DataAnnotations.Schema;
using Microsoft.EntityFrameworkCore;

namespace QCExtensions.Server.Models
{
	[Table("log_entry")]
	public class LogEntry
	{
		[Key]
		[Column("id")]
		public int Id { get; set; }

		public Guid UserToken { get; set; }

		[ForeignKey("UserToken")]
		public Token Token { get; set; }

		[Required]
		public DateTime DateTime { get; set; }

		[Required]
		public string Action { get; set; }
	}
}