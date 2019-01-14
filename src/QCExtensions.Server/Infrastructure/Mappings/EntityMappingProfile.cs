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
				.ForMember(vm => vm.Color, m => m.MapFrom((i, vm) => $"#{i.Color}"));

			CreateMap<ItemImage, ItemImageViewModel>();
		}
	}
}
