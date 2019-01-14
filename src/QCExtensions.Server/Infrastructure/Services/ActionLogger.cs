using QCExtensions.Application.Interfaces;
using QCExtensions.Domain.Entities;
using QCExtensions.Server.Models;
using System;
using System.Threading;
using System.Threading.Tasks;

namespace QCExtensions.Server.Infrastructure.Services
{
	public class ActionLogger : IActionLogger
	{
		private DomainDbContext _context;

		public ActionLogger(DomainDbContext context)
		{
			_context = context;
		}

		public async Task LogAsync(Guid token, string action, bool saveChanges = true, CancellationToken cancellationToken = default(CancellationToken))
		{
			var logEntry = new LogEntry
			{
				UserToken = token,
				DateTime = DateTime.UtcNow,
				Action = action
			};

			_context.LogEntries.Add(logEntry);
			if (saveChanges)
			{
				await _context.SaveChangesAsync(cancellationToken);
			}
		}
	}
}