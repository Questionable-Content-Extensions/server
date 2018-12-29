using System;
using AutoMapper;
using QCExtensions.Domain.Entities;
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
				.ForMember(vm => vm.Color, m => m.ResolveUsing(i => $"#{i.Color}"));

			CreateMap<ItemImage, ItemImageViewModel>();

			CreateMap<Comic, ComicViewModel>()
				.ForMember(vm => vm.Comic, m => m.MapFrom(c => c.Id))
				.ForMember(vm => vm.HasData, m => m.UseValue(true));
		}
	}
}
