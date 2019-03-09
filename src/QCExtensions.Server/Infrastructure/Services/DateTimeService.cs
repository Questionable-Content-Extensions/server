using System;
using QCExtensions.Application.Interfaces;

namespace QCExtensions.Server.Infrastructure.Services
{
	public class DateTimeService : IDateTime
	{
		public DateTime Now => DateTime.UtcNow;
	}
}
