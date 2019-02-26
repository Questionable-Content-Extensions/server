using MediatR;
using Microsoft.AspNetCore.Mvc;
using Microsoft.EntityFrameworkCore;
using QCExtensions.Application.Interfaces;
using QCExtensions.Application.Logs.Queries.GetLogs;
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
		private const int PageSize = 10;

		private readonly IMediator _mediator;

		public LogController(
			IMediator mediator
		)
		{
			this._mediator = mediator;
		}

		[HttpGet("")]
		public async Task<IActionResult> Get(Guid token, int page = 1)
			=> Ok(await _mediator.Send(new GetLogsQuery
			{
				Token = token,
				Page = page,
				PageSize = PageSize
			}));
	}
}
