using System.Linq;
using System.Security.Claims;
using System.Threading.Tasks;
using AutoMapper;
using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Identity;
using Microsoft.AspNetCore.Mvc;
using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.Options;
using Newtonsoft.Json;
using QCExtensions.Server.Models;
using QCExtensions.Server.Models.ViewModels;

namespace QCExtensions.Server.Controllers
{
	[Route("api/[controller]")]
	[ApiExplorerSettings(IgnoreApi = false, GroupName = "Item")]
	public class ItemController : Controller
	{
		private readonly IMapper _mapper;
		private readonly ApplicationDbContext _applicationDbContext;

		public ItemController(
			IMapper mapper,
			ApplicationDbContext applicationDbContext)
		{
			_mapper = mapper;
			_applicationDbContext = applicationDbContext;
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

		[HttpGet("")]
		public async Task<IActionResult> GetAll()
		{
			var items = await _applicationDbContext.Items.ToArrayAsync();
			return Ok(items);
		}
	}
}
