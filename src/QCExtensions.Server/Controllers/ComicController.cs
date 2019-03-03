using MediatR;
using Microsoft.AspNetCore.Mvc;
using QCExtensions.Application.Comics.Commands.AddItemToComic;
using QCExtensions.Application.Comics.Commands.RemoveItemFromComic;
using QCExtensions.Application.Comics.Commands.SetFlag;
using QCExtensions.Application.Comics.Commands.SetPublishDate;
using QCExtensions.Application.Comics.Commands.SetTagline;
using QCExtensions.Application.Comics.Commands.SetTitle;
using QCExtensions.Application.Comics.Models;
using QCExtensions.Application.Comics.Queries.GetAllComics;
using QCExtensions.Application.Comics.Queries.GetComic;
using QCExtensions.Server.Extensions;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Net;
using System.Threading.Tasks;

namespace QCExtensions.Server.Controllers
{
	[Route("api/[controller]")]
	[Route("api/comicdata")] // Compatibility with old server
	[ApiExplorerSettings(IgnoreApi = false, GroupName = "Comic")]
	public class ComicController : Controller
	{
		private readonly IMediator _mediator;

		public ComicController(
			IMediator mediator
		)
		{
			_mediator = mediator;
		}

		private async Task<IActionResult> SendSetFlagCommandAsync(
			SetFlagCommand command,
			SetFlagCommand.FlagType flagType,
			string flagName)
		{
			command.Flag = flagType;
			await _mediator.Send(command);
			return Ok($"{flagName} flag updated for comic");
		}

		[HttpGet("")]
		[ProducesResponseType(typeof(List<ComicListDto>), (int)HttpStatusCode.OK)]
		public async Task<IActionResult> GetAll()
		{
			var exclude = Request.Query["exclude"].SingleOrDefault();

			var exclusion = Exclusion.None;
			switch (exclude)
			{
				case "guest":
					exclusion = Exclusion.Guest;
					break;

				case "non-canon":
					exclusion = Exclusion.NonCanon;
					break;
			}

			return Ok(await _mediator.Send(new GetAllComicsQuery
			{
				Exclude = exclusion
			}));
		}

		[HttpGet("{comicId}")]
		[ProducesResponseType(typeof(ComicDto), (int)HttpStatusCode.OK)]
		public async Task<IActionResult> Get(int comicId)
		{
			var exclude = Request.Query["exclude"].SingleOrDefault();
			var include = Request.Query["include"].SingleOrDefault();
			var tokenValue = Request.Query["token"].SingleOrDefault();

			Guid? tokenGuid = null;
			if (tokenValue != null)
			{
				if (Guid.TryParse(tokenValue, out var tokenGuidValue))
				{
					tokenGuid = tokenGuidValue;
				}
			}

			var exclusion = Exclusion.None;
			switch (exclude)
			{
				case "guest":
					exclusion = Exclusion.Guest;
					break;

				case "non-canon":
					exclusion = Exclusion.NonCanon;
					break;
			}

			var inclusion = include == "all" ? Inclusion.All : Inclusion.None;

			return Ok(await _mediator.Send(new GetComicQuery
			{
				ComicId = comicId,
				Exclude = exclusion,
				Include = inclusion,
				Token = tokenGuid
			}));
		}

		[HttpPost("additem")]
		[ProducesResponseType(typeof(string), (int)HttpStatusCode.OK)]
		public async Task<IActionResult> AddItem([FromBody] AddItemToComicCommand command)
		{
			await _mediator.Send(command.OrNew());
			return Ok("Item added to comic");
		}

		[HttpPost("removeitem")]
		[ProducesResponseType(typeof(string), (int)HttpStatusCode.OK)]
		public async Task<IActionResult> RemoveItem([FromBody] RemoveItemFromComicCommand command)
		{
			await _mediator.Send(command.OrNew());
			return Ok("Item removed from comic");
		}

		[HttpPost("settitle")]
		[ProducesResponseType(typeof(string), (int)HttpStatusCode.OK)]
		public async Task<IActionResult> SetTitle([FromBody] SetTitleCommand command)
		{
			await _mediator.Send(command.OrNew());
			return Ok("Title set or updated for comic");
		}

		[HttpPost("settagline")]
		[ProducesResponseType(typeof(string), (int)HttpStatusCode.OK)]
		public async Task<IActionResult> SetTagline([FromBody] SetTaglineCommand command)
		{
			await _mediator.Send(command.OrNew());
			return Ok("Tagline set or updated for comic");
		}

		[HttpPost("setpublishdate")]
		[ProducesResponseType(typeof(string), (int)HttpStatusCode.OK)]
		public async Task<IActionResult> SetPublishDate([FromBody] SetPublishDateCommand command)
		{
			await _mediator.Send(command.OrNew());
			return Ok("Publish date set or updated for comic");
		}

		[HttpPost("setguest")]
		public async Task<IActionResult> SetGuest([FromBody] SetFlagCommand command)
			=> await SendSetFlagCommandAsync(command.OrNew(), SetFlagCommand.FlagType.IsGuestComic, "Guest comic");

		[HttpPost("setnoncanon")]
		public async Task<IActionResult> SetNonCanon([FromBody] SetFlagCommand command)
			=> await SendSetFlagCommandAsync(command.OrNew(), SetFlagCommand.FlagType.IsNonCanon, "Non-canon");

		[HttpPost("setnocast")]
		public async Task<IActionResult> SetNoCast([FromBody] SetFlagCommand command)
			=> await SendSetFlagCommandAsync(command.OrNew(), SetFlagCommand.FlagType.HasNoCast, "No cast");

		[HttpPost("setnolocation")]
		public async Task<IActionResult> SetNoLocation([FromBody] SetFlagCommand command)
			=> await SendSetFlagCommandAsync(command.OrNew(), SetFlagCommand.FlagType.HasNoLocation, "No location");

		[HttpPost("setnostoryline")]
		public async Task<IActionResult> SetNoStoryline([FromBody] SetFlagCommand command)
			=> await SendSetFlagCommandAsync(command.OrNew(), SetFlagCommand.FlagType.HasNoStoryline, "No storyline");

		[HttpPost("setnotitle")]
		public async Task<IActionResult> SetNoTitle([FromBody] SetFlagCommand command)
			=> await SendSetFlagCommandAsync(command.OrNew(), SetFlagCommand.FlagType.HasNoTitle, "No title");

		[HttpPost("setnotagline")]
		public async Task<IActionResult> SetNoTagline([FromBody] SetFlagCommand command)
			=> await SendSetFlagCommandAsync(command.OrNew(), SetFlagCommand.FlagType.HasNoTagline, "No tagline");
	}
}