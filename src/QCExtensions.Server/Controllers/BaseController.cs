using System;
using System.Threading.Tasks;
using Microsoft.AspNetCore.Mvc;
using QCExtensions.Server.Infrastructure.Services;
using QCExtensions.Server.Models.ViewModels.Results;

namespace QCExtensions.Server.Controllers
{
	public abstract class BaseController : Controller
	{
		private readonly ITokenHandler _tokenHandler;

		public BaseController(
			ITokenHandler tokenHandler)
		{
			_tokenHandler = tokenHandler;
		}

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
	}
}