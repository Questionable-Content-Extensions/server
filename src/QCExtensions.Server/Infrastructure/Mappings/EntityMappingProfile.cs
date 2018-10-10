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
		}
	}
}
