using Microsoft.AspNetCore.Mvc;
using Microsoft.EntityFrameworkCore;
using QCExtensions.Application.Interfaces;
using QCExtensions.Domain.Entities;
using QCExtensions.Server.Models.ViewModels;
using QCExtensions.Server.Models.ViewModels.Results;
using System;
using System.Linq;
using System.Threading.Tasks;

namespace QCExtensions.Server.Controllers
{
	[Route("api/[controller]")]
	[ApiExplorerSettings(IgnoreApi = false, GroupName = "Comic")]
	public class LogController : Controller
	{
		private readonly DomainDbContext _applicationDbContext;
		private readonly ITokenValidator _tokenHandler;

		public LogController(
			DomainDbContext applicationDbContext,
			ITokenValidator tokenHandler)
		{
			this._applicationDbContext = applicationDbContext;
			this._tokenHandler = tokenHandler;
		}

		private const int PageSize = 25;

		protected async Task<IActionResult> ValidateModelThen(Func<Task<IActionResult>> whenValid)
		{
			if (!ModelState.IsValid)
			{
				return BadRequest(new ModelStateErrorViewModel(ModelState));
			}

			return await whenValid();
		}

		protected async Task<IActionResult> ValidateModelAndTokenThen(Guid token, Func<Task<IActionResult>> whenValid)
			=> await ValidateModelThen(async () =>
			{
				if (!await _tokenHandler.IsValidAsync(token))
				{
					return Unauthorized();
				}

				return await whenValid();
			});

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