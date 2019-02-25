using QCExtensions.Application.Interfaces.Mapping;
using QCExtensions.Domain.Entities;

namespace QCExtensions.Application.Items.Models
{
	public class ItemDto : ItemDtoBase, IMapFrom<Item>
	{
		public int? First { get; set; }
		public int? Last { get; set; }
		public int Appearances { get; set; }
		public int TotalComics { get; set; }
		public double Presence
		{
			get
			{
				return TotalComics == 0
				? 0
				: Appearances * 100.0 / TotalComics;
			}
		}
		public bool HasImage { get; set; }
	}
}
