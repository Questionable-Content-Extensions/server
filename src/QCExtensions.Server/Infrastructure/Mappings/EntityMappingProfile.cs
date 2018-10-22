using System;
using AutoMapper;
using QCExtensions.Server.Models;
using QCExtensions.Server.Models.ViewModels;

namespace QCExtensions.Server.Infrastructure.Mappings
{
	public class EntityMappingProfile : Profile
	{
		public EntityMappingProfile()
		{
			CreateMap<Item, ItemViewModel>();
			CreateMap<Item, ItemWithTypeViewModel>();
			CreateMap<Item, ItemWithNavigationData>()
				.ForMember(i => i.Color, m => m.ResolveUsing(vm => $"#{vm.Color}"));

			CreateMap<ItemImage, ItemImageViewModel>();

			CreateMap<Comic, ComicViewModel>()
				.ForMember(vm => vm.Comic, m => m.MapFrom(c => c.Id))
				.ForMember(vm => vm.HasData, m => m.UseValue(true))
				.ForMember(vm => vm.PublishDate,
					m => m.ResolveUsing(c => c.PublishDate.HasValue
						? DateTime.SpecifyKind(c.PublishDate.Value, DateTimeKind.Utc)
						: (DateTime?)null));
		}
	}
}
