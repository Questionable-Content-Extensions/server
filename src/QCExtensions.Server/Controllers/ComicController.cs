using System;
using System.Collections.Generic;
using System.Linq;
using System.Linq.Expressions;
using System.Threading.Tasks;
using AutoMapper;
using Microsoft.AspNetCore.Mvc;
using Microsoft.EntityFrameworkCore;
using QCExtensions.Domain.Entities;
using QCExtensions.Server.Extensions.DbContext;
using QCExtensions.Server.Infrastructure.Services;
using QCExtensions.Server.Models.ViewModels;
using QCExtensions.Server.Models;
using QCExtensions.Server.Extensions;

namespace QCExtensions.Server.Controllers
{
	[Route("api/[controller]")]
	[Route("api/comicdata")] // Compatibility with old server
	[ApiExplorerSettings(IgnoreApi = false, GroupName = "Comic")]
	public class ComicController : BaseController
	{
		private const int CreateNewItemId = -1;

		private readonly IMapper _mapper;
		private readonly ApplicationDbContext _applicationDbContext;
		private readonly ITokenHandler _tokenHandler;
		private readonly IActionLogger _actionLogger;
		private readonly INewsUpdater _newsUpdater;

		public ComicController(
			IMapper mapper,
			ApplicationDbContext applicationDbContext,
			ITokenHandler tokenHandler,
			IActionLogger actionLogger,
			INewsUpdater newsUpdater)
			: base(tokenHandler)
		{
			_mapper = mapper;
			_applicationDbContext = applicationDbContext;
			_tokenHandler = tokenHandler;
			_actionLogger = actionLogger;
			_newsUpdater = newsUpdater;
		}

		private async Task<EditorData> GetEditorDataForComicAsync(int comicId)
		{
			var comicEditorData = await _applicationDbContext.QueryComicEditorData(comicId).ToListAsync();

			NavigationData castNavigationData = null, locationNavigationData = null,
				storylineNavigationData = null, titleNavigationData = null, taglineNavigationData = null;
			foreach (var comicEditorValue in comicEditorData)
			{
				switch (comicEditorValue.Type)
				{
					case "cast":
						castNavigationData = _mapper.Map<NavigationData>(comicEditorValue);
						break;

					case "location":
						locationNavigationData = _mapper.Map<NavigationData>(comicEditorValue);
						break;

					case "storyline":
						storylineNavigationData = _mapper.Map<NavigationData>(comicEditorValue);
						break;

					case "title":
						titleNavigationData = _mapper.Map<NavigationData>(comicEditorValue);
						break;

					case "tagline":
						taglineNavigationData = _mapper.Map<NavigationData>(comicEditorValue);
						break;

					default:
						throw new InvalidOperationException($"ComicEditorData stored procedure returned unexpected navigation type '{comicEditorValue.Type}'");
				}
			}

			var editorData = new EditorData
			{
				Missing = new MissingNavigationData
				{
					Cast = castNavigationData,
					Location = locationNavigationData,
					Storyline = storylineNavigationData,
					Title = titleNavigationData,
					Tagline = taglineNavigationData
				}
			};

			return editorData;
		}

		private async Task<Comic> GetOrCreateComicAsync(int comicId)
		{
			var comic = await _applicationDbContext.Comics.GetByIdAsync(comicId);
			if (comic == null)
			{
				comic = new Comic { Id = comicId };
				_applicationDbContext.Comics.Add(comic);
				await _applicationDbContext.SaveChangesAsync();
			}

			return comic;
		}

		private async Task<IActionResult> SetFlag(
			ComicBoolValueViewModel model,
			Expression<Func<Comic, bool>> flagProperty,
			string trueValueLogText,
			string falseValueLogText,
			string flagName)
			=> await ValidateModelAndTokenThen(model.Token, async () =>
			{
				var comic = await GetOrCreateComicAsync(model.Comic);
				comic.SetPropertyValue(flagProperty, model.Value);
				await _applicationDbContext.SaveChangesAsync();

				await _actionLogger.LogAsync(model.Token, $"Set comic #{model.Comic} {(model.Value ? trueValueLogText : falseValueLogText)}");

				return Ok($"{flagName} flag updated for comic");
			});

		[HttpGet("")]
		public async Task<IActionResult> GetAll()
		{
			var comics = await _applicationDbContext.Comics.Select(c => new { c.Id, c.Title }).ToArrayAsync();
			return Ok(comics);
		}

		[HttpGet("{comicId}")]
		public async Task<IActionResult> Get(int comicId)
		{
			var exclude = Request.Query["exclude"].SingleOrDefault();
			var include = Request.Query["include"].SingleOrDefault();
			var tokenValue = Request.Query["token"].SingleOrDefault();
			bool validToken = false;
			if (tokenValue != null && Guid.TryParse(tokenValue, out Guid tokenGuid))
			{
				validToken = await _tokenHandler.IsValidAsync(tokenGuid);
			}

			var comic = await _applicationDbContext.Comics.GetByIdAsync(comicId, includeItems: true, includeNews: true);
			ComicViewModel comicVM;
			if (comic != null)
			{
				comicVM = _mapper.Map<ComicViewModel>(comic);
			}
			else
			{
				comicVM = new ComicViewModel
				{
					Comic = comicId,
					HasData = false
				};
			}

			var previousQuery = _applicationDbContext.Comics.Where(c => c.Id < comicVM.Comic);
			var nextQuery = _applicationDbContext.Comics.Where(c => c.Id > comicVM.Comic);
			if (exclude == "guest")
			{
				previousQuery = previousQuery.Where(c => !c.IsGuestComic);
				nextQuery = nextQuery.Where(c => !c.IsGuestComic);
			}
			else if (exclude == "non-canon")
			{
				previousQuery = previousQuery.Where(c => !c.IsNonCanon);
				nextQuery = nextQuery.Where(c => !c.IsNonCanon);
			}
			comicVM.Previous = (await previousQuery.OrderByDescending(c => c.Id).Select(c => new { c.Id }).FirstOrDefaultAsync())?.Id;
			comicVM.Next = (await nextQuery.OrderBy(c => c.Id).Select(c => new { c.Id }).FirstOrDefaultAsync())?.Id;

			_newsUpdater.CheckFor(comicId);
			if (comic != null && comic.News != null)
			{
				comicVM.News = comic.News.NewsText;
			}

			if (validToken)
			{
				comicVM.EditorData = await GetEditorDataForComicAsync(comicId);
			}

			var comicItems = comic?.Occurrences.Select(o => o.Item);
			comicVM.Items = comicItems != null
				? _mapper.Map<ItemWithNavigationData[]>(comicItems)
				: new ItemWithNavigationData[0];

			Dictionary<int, ComicItemNavigationData> comicItemNavigationData;
			if (include == "all")
			{
				comicItemNavigationData = await _applicationDbContext.QueryComicAllItemNavigationData(comicId, exclude).ToDictionaryAsync(n => n.Id);
			}
			else
			{
				comicItemNavigationData = comicVM.Items.Length > 0
				? await _applicationDbContext.QueryComicItemNavigationData(comicId, exclude).ToDictionaryAsync(n => n.Id)
				: new Dictionary<int, ComicItemNavigationData>();
			}
			foreach (var item in comicVM.Items)
			{
				_mapper.Map(comicItemNavigationData[item.Id], item);
			}
			Array.Sort(comicVM.Items.Select(i => i.Count).ToArray(), comicVM.Items);
			Array.Reverse(comicVM.Items);

			if (include == "all")
			{
				var itemsNotInComic = _applicationDbContext.Items.Except(comicItems);
				comicVM.AllItems = _mapper.Map<ItemWithNavigationData[]>(itemsNotInComic);
				foreach (var item in comicVM.AllItems)
				{
					_mapper.Map(comicItemNavigationData[item.Id], item);
				}
				Array.Sort(comicVM.AllItems.Select(i => i.Count).ToArray(), comicVM.AllItems);
				Array.Reverse(comicVM.AllItems);
			}

			return Ok(comicVM);
		}

		[HttpPost("additem")]
		public async Task<IActionResult> AddItem([FromBody] ComicItemAssociationViewModel model)
			=> await ValidateModelAndTokenThen(model.Token, async () =>
			{
				var comic = await GetOrCreateComicAsync(model.Comic);
				var itemData = model.Item;
				Item item;
				if (itemData.Id == CreateNewItemId)
				{
					item = new Item
					{
						Name = itemData.Name,
						ShortName = itemData.Name,
						Type = itemData.Type
					};

					_applicationDbContext.Items.Add(item);
					await _applicationDbContext.SaveChangesAsync();
					await _actionLogger.LogAsync(model.Token, $"Created {item.Type} #{item.Id} ({item.Name})");
				}
				else
				{
					item = await _applicationDbContext.Items.GetByIdAsync(itemData.Id);
					if (item == null)
					{
						return BadRequest("Item does not exist");
					}

					if (await _applicationDbContext.Occurrences.ExistsAsync(comic.Id, item.Id))
					{
						return BadRequest("Item is already added to comic");
					}
				}

				_applicationDbContext.Occurrences.Add(new Occurrence { Comic = comic, Item = item });
				await _applicationDbContext.SaveChangesAsync();

				await _actionLogger.LogAsync(model.Token, $"Added {item.Type} #{item.Id} ({item.Name}) to comic #{comic.Id}");

				return Ok("Item added to comic");
			});

		[HttpPost("removeitem")]
		public async Task<IActionResult> RemoveItem([FromBody] ComicItemAssociationViewModel model)
			=> await ValidateModelAndTokenThen(model.Token, async () =>
			{
				if (!await _applicationDbContext.Comics.ExistsAsync(model.Comic))
				{
					return BadRequest("Comic does not exist");
				}
				var item = await _applicationDbContext.Items.GetByIdAsync(model.Item.Id);
				if (item == null)
				{
					return BadRequest("Item does not exist");
				}

				var occurrence = await _applicationDbContext.Occurrences.GetByComicIdAndItemIdAsync(model.Comic, item.Id);
				if (occurrence == null)
				{
					return BadRequest("Item is not in comic");
				}

				_applicationDbContext.Occurrences.Remove(occurrence);
				await _applicationDbContext.SaveChangesAsync();

				await _actionLogger.LogAsync(model.Token, $"Removed {item.Type} #{item.Id} ({item.Name}) from comic #{model.Comic}");

				return Ok("Item removed from comic");
			});

		[HttpPost("settitle")]
		public async Task<IActionResult> SetTitle([FromBody] ComicTitleViewModel model)
			=> await ValidateModelAndTokenThen(model.Token, async () =>
			{
				var comic = await _applicationDbContext.Comics.GetByIdAsync(model.Comic);
				if (comic == null)
				{
					return BadRequest();
				}

				var oldTitle = comic.Title;
				comic.Title = model.Title;
				await _applicationDbContext.SaveChangesAsync();

				if (string.IsNullOrEmpty(oldTitle))
				{
					await _actionLogger.LogAsync(model.Token, $"Set title on comic #{model.Comic} to \"{model.Title}\"");
				}
				else
				{
					await _actionLogger.LogAsync(model.Token, $"Changed title on comic #{model.Comic} from \"{oldTitle}\" to \"{model.Title}\"");
				}

				return Ok("Title set or updated for comic");
			});

		[HttpPost("settagline")]
		public async Task<IActionResult> SetTagline([FromBody] ComicTaglineViewModel model)
			=> await ValidateModelAndTokenThen(model.Token, async () =>
			{
				var comic = await _applicationDbContext.Comics.GetByIdAsync(model.Comic);
				if (comic == null)
				{
					return BadRequest();
				}

				var oldTagline = comic.Tagline;
				comic.Tagline = model.Tagline;
				await _applicationDbContext.SaveChangesAsync();

				if (string.IsNullOrEmpty(oldTagline))
				{
					await _actionLogger.LogAsync(model.Token, $"Set tagline on comic #{model.Comic} to \"{model.Tagline}\"");
				}
				else
				{
					await _actionLogger.LogAsync(model.Token, $"Changed tagline on comic #{model.Comic} from \"{oldTagline}\" to \"{model.Tagline}\"");
				}

				return Ok("Tagline set or updated for comic");
			});

		[HttpPost("setpublishdate")]
		public async Task<IActionResult> SetPublishDate([FromBody] ComicPublishDateViewModel model)
			=> await ValidateModelAndTokenThen(model.Token, async () =>
			{
				var comic = await GetOrCreateComicAsync(model.Comic);
				var oldPublishDate = comic.PublishDate;
				comic.PublishDate = model.PublishDate;
				comic.IsAccuratePublishDate = model.IsAccuratePublishDate;
				await _applicationDbContext.SaveChangesAsync();

				if (!oldPublishDate.HasValue)
				{
					await _actionLogger.LogAsync(model.Token, $"Set publish date on comic #{model.Comic} to \"{model.PublishDate:s}Z\"");
				}
				else
				{
					await _actionLogger.LogAsync(model.Token, $"Changed publish date on comic #{model.Comic} from \"{oldPublishDate:s}Z\" to \"{model.PublishDate:s}Z\"");
				}

				return Ok("Publish date set or updated for comic");
			});

		[HttpPost("setguest")]
		public async Task<IActionResult> SetGuest([FromBody] ComicBoolValueViewModel model)
			=> await SetFlag(model, c => c.IsGuestComic, "to be a guest comic", "to be a Jeph comic", "Guest comic");

		[HttpPost("setnoncanon")]
		public async Task<IActionResult> SetNonCanon([FromBody] ComicBoolValueViewModel model)
			=> await SetFlag(model, c => c.IsNonCanon, "to be non-canon", "to be canon", "Non-canon");

		[HttpPost("setnocast")]
		public async Task<IActionResult> SetNoCast([FromBody] ComicBoolValueViewModel model)
			=> await SetFlag(model, c => c.HasNoCast, "to have no cast", "to have cast", "No cast");

		[HttpPost("setnolocation")]
		public async Task<IActionResult> SetNoLocation([FromBody] ComicBoolValueViewModel model)
			=> await SetFlag(model, c => c.HasNoLocation, "to have no locations", "to have locations", "No location");

		[HttpPost("setnostoryline")]
		public async Task<IActionResult> SetNoStoryline([FromBody] ComicBoolValueViewModel model)
			=> await SetFlag(model, c => c.HasNoStoryline, "to have no storylines", "to have storylines", "No storyline");

		[HttpPost("setnotitle")]
		public async Task<IActionResult> SetNoTitle([FromBody] ComicBoolValueViewModel model)
			=> await SetFlag(model, c => c.HasNoTitle, "to have no title", "to have a title", "No title");

		[HttpPost("setnotagline")]
		public async Task<IActionResult> SetNoTagline([FromBody] ComicBoolValueViewModel model)
			=> await SetFlag(model, c => c.HasNoTagline, "to have no tagline", "to have a tagline", "No tagline");
	}
}