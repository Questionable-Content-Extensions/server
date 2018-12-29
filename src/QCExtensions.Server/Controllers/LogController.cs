using System;
using System.Linq;
using System.Threading.Tasks;
using Microsoft.AspNetCore.Mvc;
using Microsoft.EntityFrameworkCore;
using QCExtensions.Server.Infrastructure.Services;
using QCExtensions.Server.Models.ViewModels;
using QCExtensions.Server.Models;

namespace QCExtensions.Server.Controllers
{
	[Route("api/[controller]")]
	[ApiExplorerSettings(IgnoreApi = false, GroupName = "Comic")]
	public class LogController : BaseController
	{
		private readonly ApplicationDbContext _applicationDbContext;

		public LogController(
			ApplicationDbContext applicationDbContext,
			ITokenHandler tokenHandler) : base(tokenHandler)
		{
			this._applicationDbContext = applicationDbContext;
		}

		private const int PageSize = 25;

		[HttpGet("")]
		public async Task<IActionResult> Get(Guid token, int page = 1)
		{
			return await ValidateModelAndTokenThen(token, async () =>
			{
				if (page < 1)
				{
					return BadRequest("Page must be 1 or higher");
				}

				var logEntries = await _applicationDbContext.LogEntries
					.Include(l => l.Token)
					.OrderByDescending(l => l.DateTime)
					.Skip((page - 1) * PageSize)
					.Take(PageSize)
					.Select(l => new LogEntryViewModel { Identifier = l.Token.Identifier, DateTime = l.DateTime, Action = l.Action }).ToArrayAsync();
				var logEntryCount = await _applicationDbContext.LogEntries.CountAsync();
				return Ok(new LogEntriesViewModel
				{
					LogEntries = logEntries,
					Page = page,
					LogEntryCount = logEntryCount,
					PageCount = (int)Math.Ceiling(logEntryCount * 1.0 / PageSize)
				});
			});
		}
	}
}