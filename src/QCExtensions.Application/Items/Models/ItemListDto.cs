using QCExtensions.Application.Interfaces.Mapping;
using QCExtensions.Domain.Entities;

namespace QCExtensions.Application.Items.Models
{
	public class ItemListDto : ItemDtoBase, IMapFrom<ComicItemNavigationData>, IMapFrom<Item>
	{
		public int Count { get; set; }
	}
}
