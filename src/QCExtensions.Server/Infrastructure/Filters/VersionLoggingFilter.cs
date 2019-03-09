using Microsoft.AspNetCore.Mvc.Filters;
using Microsoft.Extensions.Logging;

namespace QCExtensions.Server.Infrastructure.Filters
{
	public class VersionLoggingFilter : IActionFilter
	{
		ILogger<VersionLoggingFilter> _logger;

		public VersionLoggingFilter(ILogger<VersionLoggingFilter> logger)
		{
			_logger = logger;
		}

		public void OnActionExecuted(ActionExecutedContext context)
		{
		}

		public void OnActionExecuting(ActionExecutingContext context)
		{
			var headers = context?.HttpContext?.Request?.Headers;
			if (headers == null)
			{
				return;
			}

			if (!headers.TryGetValue("X-QCExt-Version", out var version) || version.Count == 0)
			{
				return;
			}

			_logger.LogInformation($"Using QC Extension version {string.Join(", ", version)}");
		}
	}
}
