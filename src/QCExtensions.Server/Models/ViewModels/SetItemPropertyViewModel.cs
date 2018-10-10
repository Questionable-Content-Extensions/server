using System;

namespace QCExtensions.Server.Models.ViewModels
{
	public class SetItemPropertyViewModel
	{
		public Guid Token { get; set; }
		public int Item { get; set; }
		public string Property { get; set; }
		public string Value { get; set; }
	}
}
