using System;
using System.Threading.Tasks;
using QCExtensions.Domain.Entities;
using QCExtensions.Server.Models;

namespace QCExtensions.Server.Infrastructure.Services
{
	public interface IActionLogger
	{
		Task LogAsync(Guid token, string action, bool saveChanges = true);
	}

	public class ActionLogger : IActionLogger
	{
		private ApplicationDbContext _applicationDbContext;

		public ActionLogger(ApplicationDbContext applicationDbContext)
		{
			_applicationDbContext = applicationDbContext;
		}

		public async Task LogAsync(Guid token, string action, bool saveChanges = true)
		{
			var logEntry = new LogEntry
			{
				UserToken = token,
				DateTime = DateTime.UtcNow,
				Action = action
			};

			_applicationDbContext.LogEntries.Add(logEntry);
			if (saveChanges)
			{
				await _applicationDbContext.SaveChangesAsync();
			}
		}
	}
}