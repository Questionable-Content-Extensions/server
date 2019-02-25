using MediatR;
using Microsoft.AspNetCore.Mvc;
using Microsoft.Extensions.Caching.Memory;
using QCExtensions.Application.Extensions.DbContext;
using QCExtensions.Application.Items.Commands.AddImage;
using QCExtensions.Application.Items.Commands.SetColor;
using QCExtensions.Application.Items.Commands.SetName;
using QCExtensions.Application.Items.Commands.SetShortName;
using QCExtensions.Application.Items.Models;
using QCExtensions.Application.Items.Queries.GetAllItems;
using QCExtensions.Application.Items.Queries.GetImage;
using QCExtensions.Application.Items.Queries.GetItem;
using QCExtensions.Application.Items.Queries.GetItemImages;
using QCExtensions.Application.Items.Queries.GetRelatedItems;
using QCExtensions.Domain.Enumerations;
using QCExtensions.Domain.ValueObjects;
using QCExtensions.Server.Models.ViewModels;
using QCExtensions.Server.Models.ViewModels.Results;
using System.Collections.Generic;
using System.IO;
using System.Net;
using System.Threading.Tasks;

namespace QCExtensions.Server.Controllers
{
	[Route("api/[controller]")]
	[Route("api/itemdata")] // Compatibility with old server
	[ApiExplorerSettings(IgnoreApi = false, GroupName = "Item")]
	public class ItemController : Controller
	{
		private readonly IMediator _mediator;
		private readonly IMemoryCache _memoryCache;

		public ItemController(
			IMediator mediator,
			IMemoryCache memoryCache
		)
		{
			_mediator = mediator;
			_memoryCache = memoryCache;
		}

		[HttpGet("")]
		[ProducesResponseType(typeof(List<ItemListDto>), (int)HttpStatusCode.OK)]
		public async Task<IActionResult> GetAll()
			=> Ok(await _mediator.Send(new GetAllItemsQuery()));

		[HttpGet("{id}")]
		[ProducesResponseType(typeof(ItemDto), (int)HttpStatusCode.OK)]
		[ProducesResponseType((int)HttpStatusCode.NotFound)]
		public async Task<IActionResult> Get(int id)
		{
			var result = await _mediator.Send(new GetItemQuery { ItemId = id });
			if (result == null)
			{
				return NotFound();
			}
			return Ok(result);
		}

		[HttpGet("friends/{id}")] // Compatibility with old server
		[HttpGet("{id}/friends")]
		[ProducesResponseType(typeof(List<ItemListDto>), (int)HttpStatusCode.OK)]
		[ProducesResponseType((int)HttpStatusCode.NotFound)]
		public async Task<IActionResult> Friends(int id)
		{
			var result = await _mediator.Send(new GetRelatedItemsQuery
			{
				ItemId = id,
				Type = ItemType.Cast
			});
			if (result == null)
			{
				return NotFound();
			}
			return Ok(result);
		}

		[HttpGet("locations/{id}")] // Compatibility with old server
		[HttpGet("{id}/locations")]
		[ProducesResponseType(typeof(List<ItemListDto>), (int)HttpStatusCode.OK)]
		[ProducesResponseType((int)HttpStatusCode.NotFound)]
		public async Task<IActionResult> Locations(int id)
		{
			var result = await _mediator.Send(new GetRelatedItemsQuery
			{
				ItemId = id,
				Type = ItemType.Location
			});
			if (result == null)
			{
				return NotFound();
			}
			return Ok(result);
		}

		[HttpGet("{id}/images")]
		[ProducesResponseType(typeof(List<ItemImageDto>), (int)HttpStatusCode.OK)]
		[ProducesResponseType((int)HttpStatusCode.NotFound)]
		public async Task<IActionResult> ItemImages(int id)
		{
			var result = await _mediator.Send(new GetItemImagesQuery { ItemId = id });
			if (result == null)
			{
				return NotFound();
			}
			return Ok(result);
		}

		[HttpGet("image/{id}")]
		[ProducesResponseType((int)HttpStatusCode.OK)]
		[ProducesResponseType((int)HttpStatusCode.NotFound)]
		public async Task<IActionResult> Image(int id)
		{
			var itemImage = await _memoryCache.GetOrCreateAsync(id,
				async _ => await _mediator.Send(new GetImageQuery { ImageId = id }));

			if (itemImage == null)
			{
				_memoryCache.Remove(id);
				return NotFound();
			}

			return File(itemImage, "image/png");
		}

		[HttpPost("image/upload")]
		public async Task<IActionResult> UploadImage([FromForm] ItemImageUploadViewModel model)
		{
			byte[] imageData = null;
			using (var memoryStream = new MemoryStream())
			{
				await model.Image.CopyToAsync(memoryStream);
				imageData = memoryStream.GetBuffer();
			}
			await _mediator.Send(new AddImageCommand
			{
				ItemId = model.ItemId,
				Token = model.Token,
				Image = imageData,
				ImageContentType = model.Image.ContentType,
				CRC32CHash = model.CRC32CHash
			});

			return Ok();
		}

		[HttpPost("setproperty")]
		public async Task<IActionResult> SetProperty([FromBody] SetItemPropertyViewModel model)
		{
			switch (model.Property)
			{
				case "name":
					await _mediator.Send(new SetNameCommand
					{
						Token = model.Token,
						ItemId = model.Item,
						Name = model.Value.Trim()
					});
					break;

				case "shortName":
					await _mediator.Send(new SetShortNameCommand
					{
						Token = model.Token,
						ItemId = model.Item,
						ShortName = model.Value.Trim()
					});
					break;

				case "color":
					await _mediator.Send(new SetColorCommand
					{
						Token = model.Token,
						ItemId = model.Item,
						Color = (HexRgbColor)model.Value.Trim()
					});
					break;

				default:
					return BadRequest(new ErrorViewModel(new ResultViewModelBase.AssociatedError("property", "No property named " + model.Property)));
			}

			return Ok($"Item property {model.Property} has been updated on item #{model.Item}");
		}
	}
}
