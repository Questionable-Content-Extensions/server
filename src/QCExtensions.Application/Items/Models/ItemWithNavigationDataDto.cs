using AutoMapper;
using QCExtensions.Application.Interfaces.Mapping;
using QCExtensions.Domain.Entities;

namespace QCExtensions.Application.Items.Models
{
	public class ItemWithNavigationDataDto : ItemDtoBase, IHaveCustomMapping, IMapFrom<ComicItemNavigationData>
	{
		public int? First { get; set; }
		public int? Previous { get; set; }
		public int? Next { get; set; }
		public int? Last { get; set; }

		public int? Count { get; set; }

		public void CreateMappings(Profile configuration)
		{
			configuration.CreateMap<Item, ItemWithNavigationDataDto>()
				.ForMember(vm => vm.Color, m => m.MapFrom((i, vm) => $"#{i.Color}"));
		}
	}
}
