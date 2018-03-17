using System.Collections.Generic;
using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;

namespace QCExtensions.Server.Controllers
{
	public class HomeController : ControllerBase
	{
		public IActionResult Spa()
		{
			return File("~/index.html", "text/html");
		}
	}
}
