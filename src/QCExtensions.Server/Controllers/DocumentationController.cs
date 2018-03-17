using Microsoft.AspNetCore.Mvc;
using Microsoft.AspNetCore.Mvc.ApiExplorer;

namespace QCExtensions.Server.Controllers
{
	[Route("api/[controller]")]
	public class DocumentationController : Controller  
	{
		private readonly IApiDescriptionGroupCollectionProvider _apiExplorer;
		public DocumentationController(IApiDescriptionGroupCollectionProvider apiExplorer)
		{
			_apiExplorer = apiExplorer;
		}

		public IActionResult Index()
		{
			return View(_apiExplorer);
		}
	}
}
