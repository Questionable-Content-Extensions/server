using System;
using AutoMapper;
using QCExtensions.Server.Models;
using QCExtensions.Server.Models.ViewModels;

namespace QCExtensions.Server.Infrastructure.Mappings
{
	public class QueryMappingProfile : Profile
	{
		public QueryMappingProfile()
		{
			CreateMap<ComicEditorData, NavigationData>();
			CreateMap<ComicItemNavigationData, ItemWithNavigationData>();
			CreateMap<ComicItemNavigationData, ItemWithTypeViewModel>();
		}
	}
}
