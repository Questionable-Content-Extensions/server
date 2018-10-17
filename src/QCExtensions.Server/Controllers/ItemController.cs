using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Security.Claims;
using System.Threading.Tasks;
using AutoMapper;
using Force.Crc32;
using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Identity;
using Microsoft.AspNetCore.Http;
using Microsoft.AspNetCore.Mvc;
using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.Caching.Memory;
using Microsoft.Extensions.Options;
using Newtonsoft.Json;
using QCExtensions.Server.Infrastructure.Services;
using QCExtensions.Server.Models;
using QCExtensions.Server.Models.ViewModels;
using QCExtensions.Server.Models.ViewModels.Results;

namespace QCExtensions.Server.Controllers
{
	[Route("api/[controller]")]
	[Route("api/itemdata")] // Compatibility with old server
	[ApiExplorerSettings(IgnoreApi = false, GroupName = "Item")]
	public class ItemController : Controller
	{
		private readonly IMapper _mapper;
		private readonly ApplicationDbContext _applicationDbContext;
		private readonly ITokenHandler _tokenHandler;
		private readonly IActionLogger _actionLogger;
		private readonly IMemoryCache _memoryCache;

		public ItemController(
			IMapper mapper,
			ApplicationDbContext applicationDbContext,
			ITokenHandler tokenHandler,
			IActionLogger actionLogger,
			IMemoryCache memoryCache)
		{
			_mapper = mapper;
			_applicationDbContext = applicationDbContext;
			_tokenHandler = tokenHandler;
			_actionLogger = actionLogger;
			_memoryCache = memoryCache;
		}

		[HttpGet("")]
		public async Task<IActionResult> GetAll()
		{
			var items = await _applicationDbContext.Items.Select(i => new { i.Id, i.ShortName }).ToArrayAsync();
			return Ok(items);
		}

		[HttpGet("{id}")]
		public async Task<IActionResult> Get(int id)
		{
			var item = await _applicationDbContext.Items.Include(i => i.Occurrences).SingleOrDefaultAsync(i => i.Id == id);
			if (item == null)
			{
				return BadRequest();
			}

			var itemVM = _mapper.Map<ItemViewModel>(item);

			var totalComics = await _applicationDbContext.Comics.CountAsync();
			var appearances = item.Occurrences.Count;

			itemVM.TotalComics = totalComics;
			itemVM.Appearances = appearances;

			var first = item.Occurrences.Min(o => o.ComicId);
			var last = item.Occurrences.Max(o => o.ComicId);

			itemVM.First = first;
			itemVM.Last = last;

			return Ok(itemVM);
		}


		private async Task<List<ItemWithTypeViewModel>> FindTypeWith(int id, string type, int amount = 5)
		{
			var typeWithQuery = (from i in _applicationDbContext.Items
								 from o in i.Occurrences
								 join o2 in _applicationDbContext.Occurrences on o.ComicId equals o2.ComicId
								 let i2 = o2.Item
								 where i.Id == id && i2.Id != i.Id && i2.Type == type
								 group i2 by i2.Id into gi2
								 orderby gi2.Count() descending
								 select gi2)
								 .Take(amount)
								 .Select(gi2 => new
								 {
									 gi2.First().Id,
									 gi2.First().ShortName,
									 gi2.First().Name,
									 gi2.First().Type,
									 gi2.First().Color,
									 Count = gi2.Count()
								 });

			return _mapper.Map<List<ItemWithTypeViewModel>>(await typeWithQuery.ToListAsync());
		}

		[HttpGet("friends/{id}")] // Compatibility with old server
		[HttpGet("{id}/friends")]
		public async Task<IActionResult> Friends(int id)
		{
			if (!await _applicationDbContext.Items.AnyAsync(i => i.Id == id))
			{
				return BadRequest();
			}

			return Ok(await FindTypeWith(id, "cast"));
		}

		[HttpGet("locations/{id}")] // Compatibility with old server
		[HttpGet("{id}/locations")]
		public async Task<IActionResult> Locations(int id)
		{
			if (!await _applicationDbContext.Items.AnyAsync(i => i.Id == id))
			{
				return BadRequest();
			}

			return Ok(await FindTypeWith(id, "location"));
		}

		[HttpGet("{id}/images")]
		public async Task<IActionResult> ItemImages(int id)
		{
			var item = await _applicationDbContext.Items.Include(i => i.Images).SingleOrDefaultAsync(i => i.Id == id);
			if (item == null)
			{
				return BadRequest();
			}

			var viewModel = _mapper.Map<List<ItemImageViewModel>>(item.Images);
			return Ok(viewModel);
		}

		[HttpGet("image/{id}")]
		public async Task<IActionResult> Image(int id)
		{
			var itemImage = await _memoryCache.GetOrCreateAsync(id, async entry =>
			{
				var imageData = await _applicationDbContext.ItemImages.Where(i => i.Id == id).Select(i => i.Image).SingleOrDefaultAsync();
				if (imageData == null)
				{
					return null;
				}

				return imageData;
			});

			if (itemImage == null)
			{
				_memoryCache.Remove(id);
				return BadRequest();
			}

			return File(itemImage, "image/png");
		}

		[HttpPost("image/upload")]
		public async Task<IActionResult> UploadImage([FromForm] ItemImageUploadViewModel model)
		{
			if (!ModelState.IsValid)
			{
				return BadRequest(new ModelStateErrorViewModel(ModelState));
			}
			if (!await _tokenHandler.IsValidAsync(model.Token))
			{
				return Unauthorized();
			}
			if (model.Image.ContentType != "image/png")
			{
				return BadRequest("Only PNG images are supported");
			}

			byte[] imageData = null;
			using (var memoryStream = new MemoryStream())
			{
				await model.Image.CopyToAsync(memoryStream);
				imageData = memoryStream.GetBuffer();
			}

			var crc32CHash = Crc32CAlgorithm.Compute(imageData);
			if (model.CRC32CHash.HasValue && model.CRC32CHash.Value != crc32CHash)
			{
				return BadRequest("CRC32C value of image does not match with value provided in upload");
			}

			var itemImage = new ItemImage
			{
				ItemId = model.ItemId,
				CRC32CHash = crc32CHash,
				Image = imageData
			};
			_applicationDbContext.ItemImages.Add(itemImage);
			await _applicationDbContext.SaveChangesAsync();

			return Ok();
		}

		[HttpPost("setproperty")]
		public async Task<IActionResult> SetProperty([FromBody] SetItemPropertyViewModel model)
		{
			if (!ModelState.IsValid)
			{
				return BadRequest(new ModelStateErrorViewModel(ModelState));
			}
			if (!await _tokenHandler.IsValidAsync(model.Token))
			{
				return Unauthorized();
			}

			var item = await _applicationDbContext.Items.SingleOrDefaultAsync(i => i.Id == model.Item);
			if (item == null)
			{
				return BadRequest();
			}

			string oldValue = null;
			switch (model.Property)
			{
				case "name":
					oldValue = item.Name;
					item.Name = model.Value.Trim();
					break;

				case "shortName":
					oldValue = item.ShortName;
					item.ShortName = model.Value.Trim();
					break;

				case "color":
					oldValue = item.Color;
					item.Color = model.Value.Trim();
					break;

				default:
					return BadRequest(new ErrorViewModel(new ResultViewModelBase.AssociatedError("property", "No property named " + model.Property)));
			}

			_applicationDbContext.Items.Update(item);
			await _applicationDbContext.SaveChangesAsync();

			if (string.IsNullOrEmpty(oldValue))
			{
				await _actionLogger.LogAsync(model.Token, $"Set {model.Property} of {item.Type} #{item.Id} to \"{model.Value}\"");
			}
			else
			{
				await _actionLogger.LogAsync(model.Token, $"Changed {model.Property} of {item.Type} #{item.Id} from {oldValue} to \"{model.Value}\"");
			}

			return Ok($"Item property {model.Property} has been updated on item #{model.Item}");
		}
	}
}
