using System;
using Microsoft.AspNetCore.Http;

namespace QCExtensions.Server.Models.ViewModels
{
	public class ItemImageUploadViewModel
	{
		public Guid Token { get; set; }
		public int ItemId { get; set; }
		public uint? CRC32CHash { get; set; }
		public IFormFile Image { get; set; }
	}
}
