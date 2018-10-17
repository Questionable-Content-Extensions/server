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

			CreateMap<ItemImage, ItemImageViewModel>();

			CreateMap<Comic, ComicViewModel>()
				.ForMember(vm => vm.Comic, c => c.MapFrom(cs => cs.Id))
				.ForMember(vm => vm.HasData, c => c.UseValue(true))
				.ForMember(vm => vm.PublishDate,
					c => c.ResolveUsing(cs => cs.PublishDate.HasValue
						? DateTime.SpecifyKind(cs.PublishDate.Value, DateTimeKind.Utc)
						: (DateTime?)null));
		}
	}
}
