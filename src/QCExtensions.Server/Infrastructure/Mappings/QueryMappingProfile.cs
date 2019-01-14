using AutoMapper;
using QCExtensions.Domain.Entities;
using QCExtensions.Server.Models.ViewModels;

namespace QCExtensions.Server.Infrastructure.Mappings
{
	public class QueryMappingProfile : Profile
	{
		public QueryMappingProfile()
		{
			CreateMap<ComicItemNavigationData, ItemWithNavigationData>();
			CreateMap<ComicItemNavigationData, ItemWithTypeViewModel>();
		}
	}
}
