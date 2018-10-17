
using System;
using System.Linq;
using System.Threading.Tasks;
using AutoMapper;
using Microsoft.AspNetCore.Mvc;
using Microsoft.EntityFrameworkCore;
using QCExtensions.Server.Infrastructure.Services;
using QCExtensions.Server.Models;
using QCExtensions.Server.Models.ViewModels;

namespace QCExtensions.Server.Controllers
{
	[Route("api/[controller]")]
	[Route("api/comicdata")] // Compatibility with old server
	[ApiExplorerSettings(IgnoreApi = false, GroupName = "Comic")]
	public class ComicController : Controller
	{
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
		{
			_mapper = mapper;
			_applicationDbContext = applicationDbContext;
			_tokenHandler = tokenHandler;
			_actionLogger = actionLogger;
			_newsUpdater = newsUpdater;
		}

		[HttpGet("")]
		public async Task<IActionResult> GetAll()
		{
			var comics = await _applicationDbContext.Comics.Select(c => new { c.Id, c.Title }).ToArrayAsync();
			return Ok(comics);
		}

		[HttpGet("{id}")]
		public async Task<IActionResult> Get(int id)
		{
			var exclude = Request.Query["exclude"].ToArray();
			var include = Request.Query["include"].ToArray();
			var tokenValue = Request.Query["token"].SingleOrDefault();
			bool validToken = false;
			if (tokenValue != null && Guid.TryParse(tokenValue, out Guid tokenGuid))
			{
				validToken = await _tokenHandler.IsValidAsync(tokenGuid);
			}

			var comic = await _applicationDbContext.Comics.Include(c => c.Occurrences).Include(c => c.News).SingleOrDefaultAsync(c => c.Id == id);
			ComicViewModel comicVM;
			if (comic != null)
			{
				comicVM = _mapper.Map<ComicViewModel>(comic);
			}
			else
			{
				comicVM = new ComicViewModel
				{
					Comic = id,
					HasData = false
				};
			}

			_newsUpdater.CheckFor(id);
			if (comic != null && comic.News != null)
			{
				comicVM.News = comic.News.NewsText;
			}

			return Ok(comicVM);
		}
	}
}